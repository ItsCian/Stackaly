import { Link, type LinkProps, useMatchRoute } from '@tanstack/react-router';
import type * as React from 'react';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar';

// The block's version had collapsible sub-menus; the MVP only needs a flat
// list of links, so this is simpler. Sub-menus can come back later.
export function NavMain({
  items,
}: {
  items: readonly {
    title: string;
    // LinkProps['to'] = only the paths registered in the router
    to: LinkProps['to'];
    icon: React.ReactNode;
  }[];
}) {
  // useMatchRoute tells us whether a path matches the current URL
  const matchRoute = useMatchRoute();
  const { isMobile, setOpenMobile } = useSidebar();

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Navigation</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => (
          <SidebarMenuItem key={item.title}>
            <SidebarMenuButton
              // Highlights the current page. Without fuzzy matching,
              // "/dashboard" isn't active on "/dashboard/settings"
              isActive={Boolean(matchRoute({ to: item.to }))}
              // Shown when the sidebar is collapsed to icons
              tooltip={item.title}
              // Base UI: render the button as a router <Link>, so
              // navigation happens without a full page reload
              render={
                <Link
                  to={item.to}
                  // On phones the sidebar is a drawer: close it after a
                  // click, otherwise it stays open over the new page
                  onClick={() => {
                    if (isMobile) setOpenMobile(false);
                  }}
                />
              }
            >
              {item.icon}
              <span>{item.title}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
