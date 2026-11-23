import { createRootRoute, Outlet } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';
import { TooltipProvider } from '@/components/ui/tooltip';

export const Route = createRootRoute({ component: RootLayout });

// The root stays minimal: each area (home page, dashboard) brings its own
// layout, so the home page has no sidebar and the dashboard does.
function RootLayout() {
  return (
    // Tooltips need a provider above them: putting it at the root means
    // every page can use them (e.g. the collapsed sidebar's icon tooltips)
    <TooltipProvider>
      <Outlet />
      {/* Only rendered in development; removed from the production build */}
      <TanStackRouterDevtools />
    </TooltipProvider>
  );
}
