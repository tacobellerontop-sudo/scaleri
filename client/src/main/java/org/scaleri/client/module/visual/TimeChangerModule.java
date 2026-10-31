package org.scaleri.client.module.visual;

import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.NumberSetting;

public class TimeChangerModule extends Module {
    private final NumberSetting time = add(new NumberSetting("Time Of Day", 6000, 0, 23500, 500));

    public TimeChangerModule() {
        super("Time Changer", "Changes the time of day you see. Only affects you.", Category.VISUAL, false);
    }

    public long getTime() {
        return Math.round(time.get());
    }
}
