package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.util.ClickTracker;

public class CpsModule extends TextHudModule {
    private final BoolSetting showRight = add(new BoolSetting("Show Right Clicks", true));

    public CpsModule() {
        super("CPS", "Shows how many times you click per second.", true, 0.004, 0.044);
    }

    @Override
    protected String getText(boolean editing) {
        if (showRight.isOn()) {
            return ClickTracker.getLeftCps() + " | " + ClickTracker.getRightCps() + " CPS";
        }
        return ClickTracker.getLeftCps() + " CPS";
    }
}
