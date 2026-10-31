package org.scaleri.client.module.visual;

import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.option.Perspective;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.ModeSetting;

/** Look around your character in third person without changing where you're facing. */
public class FreelookModule extends Module {
    private final ModeSetting mode = add(new ModeSetting("Mode", "Hold", "Hold", "Toggle"));

    private KeyBinding key;
    private boolean active;
    private boolean toggled;
    private Perspective previous;
    private float yaw;
    private float pitch;

    public FreelookModule() {
        super("Freelook", "Hold Left Alt to look around your player. Some servers don't allow this.", Category.VISUAL, false);
    }

    public void setKey(KeyBinding key) {
        this.key = key;
    }

    public boolean isActive() {
        return active;
    }

    public float getYaw() {
        return yaw;
    }

    public float getPitch() {
        return pitch;
    }

    @Override
    public void onTick() {
        if (key == null) {
            return;
        }
        boolean want;
        if (mode.is("Toggle")) {
            while (key.wasPressed()) {
                toggled = !toggled;
            }
            want = toggled;
        } else {
            want = key.isPressed();
        }
        if (mc.player == null) {
            want = false;
        }
        if (want && !active) {
            start();
        } else if (!want && active) {
            stop();
        }
    }

    private void start() {
        active = true;
        yaw = mc.player.getYaw();
        pitch = mc.player.getPitch();
        previous = mc.options.getPerspective();
        mc.options.setPerspective(Perspective.THIRD_PERSON_BACK);
    }

    private void stop() {
        active = false;
        toggled = false;
        if (previous != null) {
            mc.options.setPerspective(previous);
        }
    }

    @Override
    protected void onDisable() {
        if (active) {
            stop();
        }
    }

    /** Mouse movement while freelooking turns the camera instead of the player. */
    public void turn(double dx, double dy) {
        yaw += (float) dx * 0.15f;
        pitch = MathHelper.clamp(pitch + (float) dy * 0.15f, -90f, 90f);
    }
}
