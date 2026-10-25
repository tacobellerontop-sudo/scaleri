package org.scaleri.client.module.pvp;

import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.setting.BoolSetting;

/**
 * Tap the sprint (or sneak) key once to keep sprinting until you tap it again,
 * the way most PvP clients do it.
 */
public class ToggleSprintModule extends TextHudModule {
    private final BoolSetting toggleSneak = add(new BoolSetting("Toggle Sneak", false));
    private final BoolSetting showHud = add(new BoolSetting("Show Status", true));

    private boolean sprintToggled;
    private boolean sneakToggled;

    public ToggleSprintModule() {
        super("Toggle Sprint", "Tap sprint once to keep sprinting. Can also toggle sneak.", Category.PVP, true, 0.004, 0.196);
        background.set(false);
    }

    /** Runs at the start of each tick, before the player reads its movement keys. */
    public void onPreTick() {
        if (!isEnabled() || mc.player == null) {
            return;
        }
        if (!mc.options.getSprintToggled().getValue()) {
            sprintToggled = handle(mc.options.sprintKey, sprintToggled);
        }
        if (toggleSneak.isOn() && !mc.options.getSneakToggled().getValue()) {
            sneakToggled = handle(mc.options.sneakKey, sneakToggled);
        } else if (sneakToggled) {
            sneakToggled = false;
            mc.options.sneakKey.setPressed(isPhysicallyDown(mc.options.sneakKey));
        }
    }

    private boolean handle(KeyBinding key, boolean toggled) {
        boolean flipped = false;
        while (key.wasPressed()) {
            flipped = !flipped;
        }
        if (flipped) {
            toggled = !toggled;
            if (!toggled) {
                key.setPressed(isPhysicallyDown(key));
            }
        }
        if (toggled && mc.currentScreen == null) {
            key.setPressed(true);
        }
        return toggled;
    }

    private boolean isPhysicallyDown(KeyBinding key) {
        InputUtil.Key bound = KeyBindingHelper.getBoundKeyOf(key);
        return bound.getCategory() == InputUtil.Type.KEYSYM
                && bound.getCode() != InputUtil.UNKNOWN_KEY.getCode()
                && InputUtil.isKeyPressed(mc.getWindow().getHandle(), bound.getCode());
    }

    @Override
    protected void onDisable() {
        if (sprintToggled) {
            mc.options.sprintKey.setPressed(isPhysicallyDown(mc.options.sprintKey));
        }
        if (sneakToggled) {
            mc.options.sneakKey.setPressed(isPhysicallyDown(mc.options.sneakKey));
        }
        sprintToggled = false;
        sneakToggled = false;
    }

    @Override
    protected String getText(boolean editing) {
        if (!showHud.isOn() && !editing) {
            return null;
        }
        if (mc.player == null) {
            return editing ? "Sprinting (Toggled)" : null;
        }
        if (mc.player.isSneaking()) {
            return sneakToggled ? "Sneaking (Toggled)" : "Sneaking";
        }
        if (sprintToggled) {
            return "Sprinting (Toggled)";
        }
        if (mc.player.isSprinting()) {
            return "Sprinting";
        }
        return editing ? "Sprinting (Toggled)" : null;
    }
}
