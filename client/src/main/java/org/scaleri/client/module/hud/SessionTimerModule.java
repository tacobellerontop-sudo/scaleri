package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;

/** How long you've been in the current world or server. */
public class SessionTimerModule extends TextHudModule {
    private long joinedAt;

    public SessionTimerModule() {
        super("Session Timer", "Shows how long you've been playing on this server.", false, 0.004, 0.434);
    }

    @Override
    public void onTick() {
        if (mc.world == null) {
            joinedAt = 0;
        } else if (joinedAt == 0) {
            joinedAt = System.currentTimeMillis();
        }
    }

    @Override
    protected String getText(boolean editing) {
        if (joinedAt == 0) {
            return editing ? "Session 0:12:34" : null;
        }
        long seconds = (System.currentTimeMillis() - joinedAt) / 1000;
        return String.format("Session %d:%02d:%02d", seconds / 3600, seconds / 60 % 60, seconds % 60);
    }
}
