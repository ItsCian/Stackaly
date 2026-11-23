import { createFileRoute } from '@tanstack/react-router';

// Placeholder page: gives the sidebar link a real destination.
// The actual screen is built in its own issue.
export const Route = createFileRoute('/dashboard/reconciliation')({
  component: Page,
});

function Page() {
  return (
    <div>
      <h1 className="text-2xl font-bold md:text-3xl">Réconciliation</h1>
      <p className="text-muted-foreground">
        Comparez les ventes et les paiements du jour.
      </p>
    </div>
  );
}
