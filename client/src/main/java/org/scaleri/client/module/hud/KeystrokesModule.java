package org.scaleri.client.module.hud;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.option.KeyBinding;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;
import org.scaleri.client.util.ClickTracker;

public class KeystrokesModule extends HudModule {
    private static final int KEY = 22;
    private static final int GAP = 2;
    private static final int WIDE = (KEY * 3 + GAP * 2 - GAP) / 2;

    private final BoolSetting showMouse = add(new BoolSetting("Show Mouse Buttons", true));
    private final BoolSetting showCps = add(new BoolSetting("Show CPS", true));
    private final BoolSetting showSpace = add(new BoolSetting("Show Spacebar", true));
    private final ColorSetting idleColor = add(new ColorSetting("Key Color", 0x6F000000));
    private final ColorSetting pressedColor = add(new ColorSetting("Pressed Color", 0xD0FFFFFF));
    private final ColorSetting textColor = add(new ColorSetting("Text Color", 0xFFFFFFFF));
    private final ColorSetting pressedTextColor = add(new ColorSetting("Pressed Text Color", 0xFF111111));

    /** Fade progress for W, A, S, D, LMB, RMB, Space. */
    private final float[] fade = new float[7];
    private long lastFrame = System.nanoTime();

    public KeystrokesModule() {
        super("Keystrokes", "Shows which movement keys and mouse buttons you're pressing.", true, 1.0, 0.02);
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        long now = System.nanoTime();
        float step = Math.min(1f, (now - lastFrame) / 1_000_000_000f * 12f);
        lastFrame = now;

        KeyBinding[] keys = {
            mc.options.forwardKey, mc.options.leftKey, mc.options.backKey, mc.options.rightKey,
            mc.options.attackKey, mc.options.useKey, mc.options.jumpKey,
        };
        for (int i = 0; i < keys.length; i++) {
            float target = keys[i].isPressed() ? 1f : 0f;
            fade[i] += (target - fade[i]) * step;
        }

        int y = 0;
        drawKey(context, KEY + GAP, y, KEY, KEY, keyName(keys[0], "W"), null, fade[0]);
        y += KEY + GAP;
        drawKey(context, 0, y, KEY, KEY, keyName(keys[1], "A"), null, fade[1]);
        drawKey(context, KEY + GAP, y, KEY, KEY, keyName(keys[2], "S"), null, fade[2]);
        drawKey(context, (KEY + GAP) * 2, y, KEY, KEY, keyName(keys[3], "D"), null, fade[3]);
        y += KEY + GAP;
        if (showMouse.isOn()) {
            String left = showCps.isOn() ? ClickTracker.getLeftCps() + " CPS" : null;
            String right = showCps.isOn() ? ClickTracker.getRightCps() + " CPS" : null;
            drawKey(context, 0, y, WIDE, KEY, "LMB", left, fade[4]);
            drawKey(context, WIDE + GAP, y, KEY * 3 + GAP * 2 - WIDE - GAP, KEY, "RMB", right, fade[5]);
            y += KEY + GAP;
        }
        if (showSpace.isOn()) {
            drawSpace(context, y, fade[6]);
            y += 12 + GAP;
        }
        width = KEY * 3 + GAP * 2;
        height = y - GAP;
    }

    private void drawKey(DrawContext context, int x, int y, int w, int h, String label, String sub, float pressed) {
        Theme.roundRect(context, x, y, w, h, 3, Theme.blend(idleColor.getArgb(), pressedColor.getArgb(), pressed));
        int color = Theme.blend(textColor.getArgb(), pressedTextColor.getArgb(), pressed) | 0xFF000000;
        if (sub == null) {
            context.drawText(mc.textRenderer, label, x + (w - mc.textRenderer.getWidth(label)) / 2 + 1, y + (h - 8) / 2, color, false);
            return;
        }
        context.drawText(mc.textRenderer, label, x + (w - mc.textRenderer.getWidth(label)) / 2 + 1, y + 3, color, false);
        context.getMatrices().push();
        context.getMatrices().translate(x + w / 2f, y + 13, 0);
        context.getMatrices().scale(0.5f, 0.5f, 1);
        context.drawText(mc.textRenderer, sub, -mc.textRenderer.getWidth(sub) / 2, 0, color, false);
        context.getMatrices().pop();
    }

    private void drawSpace(DrawContext context, int y, float pressed) {
        int w = KEY * 3 + GAP * 2;
        Theme.roundRect(context, 0, y, w, 12, 3, Theme.blend(idleColor.getArgb(), pressedColor.getArgb(), pressed));
        int color = Theme.blend(textColor.getArgb(), pressedTextColor.getArgb(), pressed) | 0xFF000000;
        context.fill(w / 2 - 12, y + 5, w / 2 + 12, y + 7, color);
    }

    private static String keyName(KeyBinding key, String fallback) {
        String name = key.getBoundKeyLocalizedText().getString();
        return name.length() <= 3 ? name.toUpperCase() : fallback;
    }
}
