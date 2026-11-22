import { createFileRoute } from '@tanstack/react-router';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';

export const Route = createFileRoute('/dashboard/')({
  component: DashboardPage,
});

// The day's key figures. The values are Skeletons for now: once the API
// client exists (step 9), they get real amounts formatted by formatAmount().
const stats = [
  { title: 'Ventes du jour', description: 'Total des caisses' },
  { title: 'Paiements encaissés', description: 'Terminaux et plateformes' },
  { title: 'Écart', description: 'Ventes − paiements' },
  { title: 'Statut', description: 'Réconciliation du jour' },
];

function DashboardPage() {
  return (
    <>
      <div>
        <h1 className="text-2xl font-bold md:text-3xl">Bonjour 👋</h1>
        <p className="text-muted-foreground">
          Voici où en est la réconciliation d'aujourd'hui.
        </p>
      </div>

      {/* ── Key figures: 1 column → 2 → 4 ───────────── */}
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.title}>
            <CardHeader>
              <CardDescription>{stat.title}</CardDescription>
              {/* Figma rule "Numbers in mono": amounts use font-mono.
                  The Skeleton stands in for the number until data loads */}
              <CardTitle className="font-mono text-2xl">
                <Skeleton className="h-8 w-28" />
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-xs text-muted-foreground">
                {stat.description}
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* ── Main area: chart (wide) + latest reconciliations ── */}
      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Ventes des 7 derniers jours</CardTitle>
            <CardDescription>Caisses vs paiements encaissés</CardDescription>
          </CardHeader>
          <CardContent>
            {/* Placeholder for a future shadcn chart */}
            <Skeleton className="h-64 w-full" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Dernières réconciliations</CardTitle>
            <CardDescription>Les 5 derniers jours</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            {/* Array.from({ length: 5 }) = 5 placeholder rows.
                Using the index as key is fine here: the list is static */}
            {Array.from({ length: 5 }, (_, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: static placeholder list
              <div key={i} className="flex items-center justify-between">
                <div className="flex flex-col gap-2">
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-3 w-16" />
                </div>
                <Skeleton className="h-6 w-20 rounded-full" />
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
