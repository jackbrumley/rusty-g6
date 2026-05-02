interface TabPageHeaderProps {
  title: string;
  subtitle: string;
}

export function TabPageHeader({ title, subtitle }: TabPageHeaderProps) {
  return (
    <header class="tab-page-header">
      <h2 class="tab-page-title">{title}</h2>
      <p class="tab-page-subtitle">{subtitle}</p>
    </header>
  );
}
