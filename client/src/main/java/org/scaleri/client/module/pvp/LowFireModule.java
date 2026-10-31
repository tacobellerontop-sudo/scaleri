package org.scaleri.client.module.pvp;

import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.NumberSetting;

public class LowFireModule extends Module {
    public final NumberSetting offset = add(new NumberSetting("Lower By", 0.3, 0.0, 0.6, 0.05));

    public LowFireModule() {
        super("Low Fire", "Moves the burning overlay down so you can see while on fire.", Category.PVP, true);
    }
}
