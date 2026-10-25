package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;

public class FpsModule extends TextHudModule {
    public FpsModule() {
        super("FPS", "Shows your frames per second.", true, 0.004, 0.008);
    }

    @Override
    protected String getText(boolean editing) {
        return mc.getCurrentFps() + " FPS";
    }
}
