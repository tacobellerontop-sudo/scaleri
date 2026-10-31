package org.scaleri.client.module.visual;

import net.minecraft.client.option.KeyBinding;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.NumberSetting;

public class ZoomModule extends Module {
    private final NumberSetting amount = add(new NumberSetting("Zoom Amount", 4, 1.5, 20, 0.5));
    private final BoolSetting smooth = add(new BoolSetting("Smooth Zoom", true));
    private final BoolSetting scroll = add(new BoolSetting("Scroll To Adjust", true));
    private final BoolSetting cinematic = add(new BoolSetting("Cinematic Camera", false));

    private KeyBinding key;
    private double scrollFactor = 1;
    private double current = 1;
    private long lastFrame = System.nanoTime();
    private boolean wasZooming;
    private boolean savedCinematic;

    public ZoomModule() {
        super("Zoom", "Hold the zoom key (C) to zoom in. Scroll while zooming to adjust.", Category.VISUAL, true);
    }

    public void setKey(KeyBinding key) {
        this.key = key;
    }

    public boolean isZooming() {
        return isEnabled() && key != null && key.isPressed() && mc.currentScreen == null;
    }

    @Override
    public void onTick() {
        boolean zooming = isZooming();
        if (zooming && !wasZooming) {
            scrollFactor = 1;
            if (cinematic.isOn()) {
                savedCinematic = mc.options.smoothCameraEnabled;
                mc.options.smoothCameraEnabled = true;
            }
        } else if (!zooming && wasZooming && cinematic.isOn()) {
            mc.options.smoothCameraEnabled = savedCinematic;
        }
        wasZooming = zooming;
    }

    @Override
    protected void onDisable() {
        if (wasZooming && cinematic.isOn()) {
            mc.options.smoothCameraEnabled = savedCinematic;
        }
        wasZooming = false;
        current = 1;
    }

    /** Returns true if the scroll was used to change zoom. */
    public boolean onScroll(double vertical) {
        if (!isZooming() || !scroll.isOn() || vertical == 0) {
            return false;
        }
        scrollFactor = MathHelper.clamp(scrollFactor * (vertical > 0 ? 1.25 : 0.8), 0.25, 8);
        return true;
    }

    /** How much to divide the field of view by this frame. */
    public double getDivisor() {
        long now = System.nanoTime();
        double dt = (now - lastFrame) / 1_000_000_000.0;
        lastFrame = now;
        double target = isZooming() ? MathHelper.clamp(amount.get() * scrollFactor, 1, 50) : 1;
        if (!smooth.isOn()) {
            current = target;
        } else {
            current += (target - current) * Math.min(1, dt * 14);
            if (Math.abs(current - target) < 0.001) {
                current = target;
            }
        }
        return current;
    }
}
