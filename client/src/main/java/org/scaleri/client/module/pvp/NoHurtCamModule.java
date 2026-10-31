package org.scaleri.client.module.pvp;

import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;

public class NoHurtCamModule extends Module {
    public NoHurtCamModule() {
        super("No Hurt Cam", "Stops your screen from shaking when you take damage.", Category.PVP, true);
    }
}
