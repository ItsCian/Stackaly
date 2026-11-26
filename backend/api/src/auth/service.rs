//! Business logic for authentication. No HTTP here: takes plain values,
//! returns Result, so it can be tested without a server.

use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, NotSet,
    QueryFilter, QuerySelect, Set, TransactionTrait, entity::prelude::DateTimeWithTimeZone,
};
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    auth::{dto::LoginRequest, normalize_email, token},
    entities::{companies, memberships, sessions, users},
    errors::AppError,
};

/// How long a session stays valid after login (one working day).
pub const SESSION_DURATION_HOURS: i64 = 12;

/// What a successful login gives back to the handler.
pub struct NewSession {
    pub token: String,
    pub expires_at: DateTimeWithTimeZone,
}

/// The company the caller is currently working in, with their role in it.
pub struct ActiveCompany {
    pub id: Uuid,
    pub role: String,
}

/// The authenticated caller, given to every protected handler.
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub company: Option<ActiveCompany>,
}

impl AuthUser {
    /// Returns the caller's active company if it is the one requested.
    /// Fails with NotFound otherwise, so outsiders can't tell it exists.
    pub fn company(&self, id: Uuid) -> Result<&ActiveCompany, AppError> {
        match self.company.as_ref() {
            Some(c) if c.id == id => Ok(c),
            _ => Err(AppError::NotFound),
        }
    }

    pub fn owned_company(&self, id: Uuid) -> Result<&ActiveCompany, AppError> {
        let company = self.company(id)?;
        if company.role != "owner" {
            return Err(AppError::Forbidden("Only the owner can do this".into()));
        }

        Ok(company)
    }
}

/// Checks the credentials and opens a new session.
/// Unknown email and wrong password give the same error, in the same time,
/// so the response never reveals which accounts exist.
pub async fn login(db: &DatabaseConnection, input: LoginRequest) -> Result<NewSession, AppError> {
    let email = normalize_email(&input.email);
    let user = users::Entity::find().filter(users::Column::Email.eq(email)).one(db).await?;

    let password_hash = match &user {
        Some(u) => u.password_hash.clone(),
        None => super::password::DUMMY_HASH.clone(),
    };

    let password = input.password;
    let verification = tokio::task::spawn_blocking(move || {
        super::password::verify_password(password.expose_secret(), &password_hash)
    })
    .await;

    let password_ok = match verification {
        Ok(Ok(ok)) => ok,
        Ok(Err(err)) => return Err(AppError::Internal(format!("corrupt password hash: {err}"))),
        Err(err) => return Err(AppError::Internal(format!("password check task failed: {err}"))),
    };

    let user = match user {
        Some(u) if password_ok => u,
        _ => return Err(AppError::InvalidCredentials),
    };

    if user.disabled_at.is_some() {
        return Err(AppError::Forbidden("This account is disabled".into()));
    }

    let active_memberships = memberships::Entity::find()
        .filter(memberships::Column::UserId.eq(user.id))
        .filter(memberships::Column::DeactivatedAt.is_null())
        .limit(2)
        .all(db)
        .await?;

    let active_company_id: Option<Uuid> = match active_memberships.as_slice() {
        [only] => Some(only.company_id),
        _ => None,
    };

    let now = Utc::now();

    let txn = db.begin().await?;

    let session = create_session(&txn, user.id, active_company_id).await?;

    let mut user: users::ActiveModel = user.into();
    user.last_login_at = Set(Some(now.into()));
    user.failed_login_count = Set(0);
    user.updated_at = Set(now.into());
    user.update(&txn).await?;

    txn.commit().await?;

    Ok(session)
}

/// Opens a new session for this user: generates a random token, stores
/// only its SHA-256 hash, and returns the raw token for the cookie.
///
/// Shared by `login` and `register`. Generic over `ConnectionTrait` so the
/// caller decides whether it runs inside a transaction.
pub async fn create_session<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
    active_company_id: Option<Uuid>,
) -> Result<NewSession, AppError> {
    let raw_token = token::generate_token()
        .map_err(|err| AppError::Internal(format!("token generation failed: {err}")))?;
    let token_hash = token::hash_token(&raw_token); // only the hash is stored

    let expires_at: DateTimeWithTimeZone =
        (Utc::now() + Duration::hours(SESSION_DURATION_HOURS)).into();

    let session = sessions::ActiveModel {
        id: Set(Uuid::now_v7()),
        token_hash: Set(token_hash.to_vec()),
        user_id: Set(user_id),
        active_company_id: Set(active_company_id),
        created_at: NotSet, // DB default now()
        expires_at: Set(expires_at),
        last_seen_at: NotSet, // DB default now()
    };
    session.insert(db).await?;

    Ok(NewSession { token: raw_token, expires_at })
}

/// Deletes the session matching this raw token. Does nothing if there is none.
pub async fn logout(db: &DatabaseConnection, token: &str) -> Result<(), AppError> {
    let token_hash = token::hash_token(token);

    sessions::Entity::delete_many()
        .filter(sessions::Column::TokenHash.eq(token_hash.to_vec()))
        .exec(db)
        .await?;

    Ok(())
}

/// Checks a raw session token and returns the authenticated caller.
/// Fails with Unauthorized if the session is missing, expired or invalidated.
pub async fn authenticate(db: &DatabaseConnection, token: &str) -> Result<AuthUser, AppError> {
    let token_hash = token::hash_token(token);
    let session = sessions::Entity::find()
        .filter(sessions::Column::TokenHash.eq(token_hash.to_vec()))
        .one(db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if session.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let user =
        users::Entity::find_by_id(session.user_id).one(db).await?.ok_or(AppError::Unauthorized)?;

    if user.disabled_at.is_some() {
        return Err(AppError::Forbidden("This account is disabled".into()));
    }

    if user.password_changed_at > session.created_at {
        return Err(AppError::Unauthorized);
    }

    let company = match session.active_company_id {
        None => None,

        Some(company_id) => {
            let membership = memberships::Entity::find_by_id((user.id, company_id)).one(db).await?;
            let company = companies::Entity::find_by_id(company_id).one(db).await?;

            match (membership, company) {
                (Some(m), Some(c))
                    if m.deactivated_at.is_none()
                        && c.closed_at.is_none()
                        && c.suspended_at.is_none() =>
                {
                    Some(ActiveCompany { id: company_id, role: m.role })
                }
                _ => None,
            }
        }
    };
    Ok(AuthUser { user_id: user.id, session_id: session.id, company })
}
