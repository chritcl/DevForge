import { useUIStore, type ThemePreference } from "../stores/ui";

const THEME_OPTIONS: Array<{ value: ThemePreference; label: string }> = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

export function SettingsPage() {
  const theme = useUIStore((state) => state.theme);
  const setTheme = useUIStore((state) => state.setTheme);

  return (
    <div className="settings-page">
      <h1>设置</h1>

      <fieldset className="theme-fieldset">
        <legend>外观主题</legend>
        {THEME_OPTIONS.map((option) => (
          <label key={option.value} className="theme-option">
            <input
              type="radio"
              name="theme"
              value={option.value}
              checked={theme === option.value}
              onChange={() => setTheme(option.value)}
            />
            {option.label}
          </label>
        ))}
      </fieldset>
    </div>
  );
}
