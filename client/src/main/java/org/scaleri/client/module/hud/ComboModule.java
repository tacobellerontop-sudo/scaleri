package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.util.CombatTracker;

public class ComboModule extends TextHudModule {
    private final BoolSetting showZero = add(new BoolSetting("Show With No Combo", true));

    public ComboModule() {
        super("Combo Counter", "Counts hits you land in a row without getting hit back.", true, 1.0, 0.245);
    }

    @Override
    protected String getText(boolean editing) {
        int combo = CombatTracker.getCombo();
        if (combo == 0 && !showZero.isOn() && !editing) {
            return null;
        }
        return combo == 0 ? "No Combo" : combo + " Combo";
    }
}
