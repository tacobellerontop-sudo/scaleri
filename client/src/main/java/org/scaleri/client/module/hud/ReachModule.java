package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.util.CombatTracker;

public class ReachModule extends TextHudModule {
    public ReachModule() {
        super("Reach Display", "Shows the distance of your last hit.", true, 1.0, 0.205);
    }

    @Override
    protected String getText(boolean editing) {
        double reach = CombatTracker.getLastReach();
        return reach < 0 ? "-- blocks" : String.format("%.2f blocks", reach);
    }
}
