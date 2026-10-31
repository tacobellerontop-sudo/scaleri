package org.scaleri.client.module.hud;

import net.minecraft.client.network.ServerInfo;
import org.scaleri.client.module.TextHudModule;

public class ServerAddressModule extends TextHudModule {
    public ServerAddressModule() {
        super("Server Address", "Shows which server you are playing on.", false, 0.004, 0.398);
    }

    @Override
    protected String getText(boolean editing) {
        if (mc.isInSingleplayer()) {
            return "Singleplayer";
        }
        ServerInfo server = mc.getCurrentServerEntry();
        if (server == null) {
            return editing ? "play.example.net" : null;
        }
        return server.address;
    }
}
