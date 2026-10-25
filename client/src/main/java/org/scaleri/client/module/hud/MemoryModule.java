package org.scaleri.client.module.hud;

import org.scaleri.client.module.TextHudModule;

public class MemoryModule extends TextHudModule {
    public MemoryModule() {
        super("Memory", "Shows how much RAM the game is using.", false, 0.004, 0.362);
    }

    @Override
    protected String getText(boolean editing) {
        Runtime rt = Runtime.getRuntime();
        long used = (rt.totalMemory() - rt.freeMemory()) / (1024 * 1024);
        long max = rt.maxMemory() / (1024 * 1024);
        return String.format("RAM %d%% (%d/%d MB)", used * 100 / Math.max(1, max), used, max);
    }
}
