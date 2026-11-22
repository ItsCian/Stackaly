import { createFileRoute, Outlet } from '@tanstack/react-router';
import { AppSidebar } from '@/components/app-sidebar';
import { Separator } from '@/components/ui/separator';
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from '@/components/ui/sidebar';

// route.tsx inside a folder = the layout for every page in that folder.
// /dashboard, and later /dashboard/reconciliation, /dashboard/settings…
// all render inside <Outlet /> below, with the sidebar around them.
export const Route = createFileRoute('/dashboard')({
  component: DashboardLayout,
});

function DashboardLayout() {
  return (
    // SidebarProvider holds the open/collapsed state and the keyboard
    // shortcut (Cmd/Ctrl+B). Everything using the sidebar must be inside it.
    <SidebarProvider>
      {/* Desktop: fixed on the left, collapses to icons.
          Phone: hidden, opens as a drawer via SidebarTrigger */}
      <AppSidebar />

      {/* SidebarInset = the main area next to the sidebar */}
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
          {/* -ml-1 lines the icon up optically with the content below */}
          <SidebarTrigger className="-ml-1" />
          <Separator
            orientation="vertical"
            className="mr-2 data-[orientation=vertical]:h-4"
          />
          <span className="font-medium">Tableau de bord</span>
        </header>

        {/* The current page (e.g. dashboard/index.tsx) renders here */}
        <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
