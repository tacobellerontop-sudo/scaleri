package org.scaleri.client.module.misc;

import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;

/** Always-on module that holds client-wide look and feel settings. */
public class ThemeModule extends Module {
    public final ColorSetting accent = add(new ColorSetting("Accent Color", 0xFF748FFC));
    public final BoolSetting menuButton = add(new BoolSetting("Menu Button On Title Screen", true));
    public final BoolSetting windowTitle = add(new BoolSetting("Scaleri Window Title", true));

    public ThemeModule() {
        super("Theme", "Accent color and client branding.", Category.MISC, true);
    }

    @Override
    public boolean isToggleable() {
        return false;
    }
}
