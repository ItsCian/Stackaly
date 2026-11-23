import {
  Clock01Icon,
  DashboardSquare01Icon,
  Invoice01Icon,
  Settings05Icon,
  Store01Icon,
} from '@hugeicons/core-free-icons';
import { HugeiconsIcon } from '@hugeicons/react';
import type * as React from 'react';
import { NavMain } from '@/components/nav-main';
import { NavUser } from '@/components/nav-user';
import { TeamSwitcher } from '@/components/team-switcher';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@/components/ui/sidebar';

// Placeholder data. Once the API client exists (step 9), the user and
// their companies come from the session instead of being hardcoded.
const data = {
  user: {
    name: 'Cian Jones',
    email: 'cian@example.com',
    avatar: '',
  },
  companies: [
    {
      name: 'My Italian Friends',
      logo: <HugeiconsIcon icon={Store01Icon} strokeWidth={2} />,
      plan: 'Réconciliation',
    },
  ],
  // `to` must be a route that exists in routeTree.gen.ts: TypeScript
  // rejects a typo or a page that hasn't been created yet
  navMain: [
    {
      title: 'Tableau de bord',
      to: '/dashboard',
      icon: <HugeiconsIcon icon={DashboardSquare01Icon} strokeWidth={2} />,
    },
    {
      title: 'Réconciliation',
      to: '/dashboard/reconciliation',
      icon: <HugeiconsIcon icon={Invoice01Icon} strokeWidth={2} />,
    },
    {
      title: 'Historique',
      to: '/dashboard/history',
      icon: <HugeiconsIcon icon={Clock01Icon} strokeWidth={2} />,
    },
    {
      title: 'Paramètres',
      to: '/dashboard/settings',
      icon: <HugeiconsIcon icon={Settings05Icon} strokeWidth={2} />,
    },
  ] as const,
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    // collapsible="icon": on desktop the sidebar shrinks to its icons
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.companies} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      {/* Thin clickable strip on the sidebar's edge to toggle it */}
      <SidebarRail />
    </Sidebar>
  );
}
