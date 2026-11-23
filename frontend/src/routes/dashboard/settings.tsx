import { createFileRoute } from '@tanstack/react-router';

// Placeholder page: gives the sidebar link a real destination.
// The actual screen is built in its own issue.
export const Route = createFileRoute('/dashboard/settings')({
  component: Page,
});

function Page() {
  return (
    <div>
      <h1 className="text-2xl font-bold md:text-3xl">Paramètres</h1>
      <p className="text-muted-foreground">
        Gérez votre commerce, vos sources de vente et vos modules.
      </p>
    </div>
  );
}
