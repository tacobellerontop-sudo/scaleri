package org.scaleri.client.module.hud;

import net.minecraft.client.network.PlayerListEntry;
import org.scaleri.client.module.TextHudModule;

public class PingModule extends TextHudModule {
    public PingModule() {
        super("Ping", "Shows your latency to the server.", true, 0.004, 0.080);
    }

    @Override
    protected String getText(boolean editing) {
        if (mc.player == null || mc.getNetworkHandler() == null) {
            return editing ? "42 ms" : null;
        }
        PlayerListEntry entry = mc.getNetworkHandler().getPlayerListEntry(mc.player.getUuid());
        return (entry == null ? 0 : entry.getLatency()) + " ms";
    }
}
