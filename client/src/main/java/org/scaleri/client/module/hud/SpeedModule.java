package org.scaleri.client.module.hud;

import net.minecraft.util.math.Vec3d;
import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.setting.ModeSetting;

public class SpeedModule extends TextHudModule {
    private final ModeSetting unit = add(new ModeSetting("Unit", "Blocks/s", "Blocks/s", "km/h"));
    private Vec3d lastPos;
    private double blocksPerSecond;

    public SpeedModule() {
        super("Speed", "Shows how fast you are moving horizontally.", false, 0.004, 0.326);
    }

    @Override
    public void onTick() {
        if (mc.player == null) {
            lastPos = null;
            blocksPerSecond = 0;
            return;
        }
        Vec3d pos = mc.player.getPos();
        if (lastPos != null) {
            double dx = pos.x - lastPos.x;
            double dz = pos.z - lastPos.z;
            double instant = Math.sqrt(dx * dx + dz * dz) * 20;
            // Smooth over a few ticks so the number doesn't flicker.
            blocksPerSecond += (instant - blocksPerSecond) * 0.5;
        }
        lastPos = pos;
    }

    @Override
    protected String getText(boolean editing) {
        if (unit.is("km/h")) {
            return String.format("%.1f km/h", blocksPerSecond * 3.6);
        }
        return String.format("%.2f b/s", blocksPerSecond);
    }
}
