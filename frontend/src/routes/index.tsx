import {
  ArrowRight01Icon,
  Calendar03Icon,
  Invoice01Icon,
  PackageIcon,
  ShoppingCart01Icon,
} from '@hugeicons/core-free-icons';
import { HugeiconsIcon } from '@hugeicons/react';
import { createFileRoute, Link } from '@tanstack/react-router';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';

export const Route = createFileRoute('/')({
  component: HomePage,
});

// The modules shown on the home page. Kept as data (not JSX) so adding
// a module = adding one object. Text moves to Paraglide in step 7.
const modules = [
  {
    title: 'Réconciliation',
    description:
      'Vérifiez chaque soir que vos ventes et vos paiements concordent.',
    icon: Invoice01Icon,
    available: true,
  },
  {
    title: 'Caisse & stock',
    description: 'Suivez vos ventes et vos niveaux de stock au même endroit.',
    icon: PackageIcon,
    available: false,
  },
  {
    title: 'Réservations',
    description: 'Gérez les réservations de votre trattoria.',
    icon: Calendar03Icon,
    available: false,
  },
  {
    title: 'Commandes',
    description: 'Centralisez les commandes du traiteur et à emporter.',
    icon: ShoppingCart01Icon,
    available: false,
  },
];

function HomePage() {
  return (
    // min-h-svh: full screen height, using the "small viewport" unit so
    // mobile browser toolbars don't cause a jump
    <div className="flex min-h-svh flex-col">
      <header className="border-b">
        <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 md:px-6">
          <span className="font-heading text-xl font-bold">Stackaly</span>
          {/* Base UI's Button renders a <Link> via the `render` prop
              (Radix-based examples online use `asChild` instead).
              nativeButton={false}: tells Base UI the result is an <a>,
              not a <button>, so it sets the right accessibility attributes */}
          <Button
            variant="outline"
            nativeButton={false}
            render={<Link to="/dashboard" />}
          >
            Se connecter
          </Button>
        </div>
      </header>

      <main className="flex-1">
        {/* ── Hero ───────────────────────────────────── */}
        <section className="mx-auto max-w-6xl px-4 py-16 md:px-6 md:py-24">
          <div className="max-w-2xl">
            <h1 className="text-4xl font-extrabold tracking-tight md:text-6xl">
              Vos ventes du jour, réconciliées en quelques minutes.
            </h1>
            <p className="mt-6 text-lg text-muted-foreground">
              Stackaly compare automatiquement vos caisses, vos terminaux de
              paiement et vos plateformes de vente, et vous signale chaque
              écart.
            </p>
            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <Button
                size="lg"
                nativeButton={false}
                render={<Link to="/dashboard" />}
              >
                Ouvrir le tableau de bord
                {/* data-icon: lets the shadcn Button size/space the icon */}
                <HugeiconsIcon icon={ArrowRight01Icon} data-icon="inline-end" />
              </Button>
            </div>
          </div>
        </section>

        {/* ── Modules ────────────────────────────────── */}
        <section className="border-t bg-muted">
          <div className="mx-auto max-w-6xl px-4 py-16 md:px-6">
            <h2 className="text-2xl font-bold md:text-3xl">
              Un module par besoin
            </h2>
            <p className="mt-2 text-muted-foreground">
              Activez uniquement ce dont votre commerce a besoin.
            </p>

            {/* 1 column on phones, 2 on tablets, 4 on desktop */}
            <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {modules.map((items) => (
                <Card key={items.title}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <HugeiconsIcon
                        icon={items.icon}
                        className="size-6 text-primary"
                      />
                      {/* Status badge: success colours from the Figma tokens */}
                      {items.available ? (
                        <Badge className="bg-success-subtle text-success-text">
                          Disponible
                        </Badge>
                      ) : (
                        <Badge variant="outline">Bientôt</Badge>
                      )}
                    </div>
                    <CardTitle className="mt-4">{items.title}</CardTitle>
                    <CardDescription>{items.description}</CardDescription>
                  </CardHeader>
                </Card>
              ))}
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t">
        <div className="mx-auto max-w-6xl px-4 py-6 text-sm text-muted-foreground md:px-6">
          © {new Date().getFullYear()} Stackaly
        </div>
      </footer>
    </div>
  );
}
