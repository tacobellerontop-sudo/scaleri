package org.scaleri.client.module.pvp;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.option.AttackIndicator;
import net.minecraft.entity.LivingEntity;
import net.minecraft.util.hit.EntityHitResult;
import net.minecraft.world.GameMode;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;
import org.scaleri.client.setting.ModeSetting;
import org.scaleri.client.setting.NumberSetting;

public class CrosshairModule extends Module {
    public final ModeSetting style = add(new ModeSetting("Style", "Cross", "Cross", "Cross + Dot", "Dot", "Circle", "Square", "T Shape"));
    public final NumberSetting size = add(new NumberSetting("Size", 4, 1, 12, 1));
    public final NumberSetting gap = add(new NumberSetting("Gap", 2, 0, 8, 1));
    public final NumberSetting thickness = add(new NumberSetting("Thickness", 1, 1, 4, 1));
    public final ColorSetting color = add(new ColorSetting("Color", 0xFFFFFFFF));
    public final BoolSetting outline = add(new BoolSetting("Outline", true));
    public final ColorSetting outlineColor = add(new ColorSetting("Outline Color", 0xC0000000));
    public final BoolSetting highlightTargets = add(new BoolSetting("Highlight On Target", true));
    public final ColorSetting targetColor = add(new ColorSetting("Target Color", 0xFFFF6B6B));
    public final BoolSetting showInThirdPerson = add(new BoolSetting("Show In Third Person", false));

    public CrosshairModule() {
        super("Custom Crosshair", "Pick your own crosshair shape, size and color.", Category.PVP, false);
    }

    /** Draws the crosshair in place of the vanilla one. */
    public void renderCrosshair(DrawContext ctx) {
        if (mc.player == null || mc.interactionManager == null) {
            return;
        }
        if (mc.getDebugHud().shouldShowDebugHud()) {
            return;
        }
        if (!mc.options.getPerspective().isFirstPerson() && !showInThirdPerson.isOn()) {
            return;
        }
        if (mc.interactionManager.getCurrentGameMode() == GameMode.SPECTATOR && !(mc.crosshairTarget instanceof EntityHitResult)) {
            return;
        }

        int cx = ctx.getScaledWindowWidth() / 2;
        int cy = ctx.getScaledWindowHeight() / 2;
        boolean onTarget = highlightTargets.isOn()
                && mc.crosshairTarget instanceof EntityHitResult hit
                && hit.getEntity() instanceof LivingEntity;
        int main = onTarget ? targetColor.getArgb() : color.getArgb();

        if (outline.isOn()) {
            draw(ctx, cx, cy, outlineColor.getArgb(), 1);
        }
        draw(ctx, cx, cy, main, 0);
        renderAttackIndicator(ctx, cx, cy, main);
    }

    private void draw(DrawContext ctx, int cx, int cy, int argb, int grow) {
        int t = thickness.getInt();
        int s = size.getInt();
        int g = gap.getInt();
        int lo = -(t / 2) - grow;
        int hi = lo + t + grow * 2;
        switch (style.get()) {
            case "Dot" -> ctx.fill(cx + lo, cy + lo, cx + hi, cy + hi, argb);
            case "Square" -> {
                int r = s + g;
                ring(ctx, cx - r - grow, cy - r - grow, (r + grow) * 2 + 1, t + grow * 2, argb);
            }
            case "Circle" -> circle(ctx, cx, cy, s + g + grow, t + grow * 2, argb);
            default -> {
                boolean tShape = style.is("T Shape");
                // left / right
                ctx.fill(cx - g - s - grow, cy + lo, cx - g + grow, cy + hi, argb);
                ctx.fill(cx + g + 1 - grow, cy + lo, cx + g + s + 1 + grow, cy + hi, argb);
                // top / bottom
                if (!tShape) {
                    ctx.fill(cx + lo, cy - g - s - grow, cx + hi, cy - g + grow, argb);
                }
                ctx.fill(cx + lo, cy + g + 1 - grow, cx + hi, cy + g + s + 1 + grow, argb);
                if (style.is("Cross + Dot")) {
                    ctx.fill(cx + lo, cy + lo, cx + hi, cy + hi, argb);
                }
            }
        }
    }

    private static void ring(DrawContext ctx, int x, int y, int size, int t, int argb) {
        ctx.fill(x, y, x + size, y + t, argb);
        ctx.fill(x, y + size - t, x + size, y + size, argb);
        ctx.fill(x, y + t, x + t, y + size - t, argb);
        ctx.fill(x + size - t, y + t, x + size, y + size - t, argb);
    }

    private static void circle(DrawContext ctx, int cx, int cy, int radius, int t, int argb) {
        double outer = radius + 0.5;
        double inner = radius + 0.5 - t;
        for (int dy = -radius; dy <= radius; dy++) {
            int runStart = Integer.MIN_VALUE;
            for (int dx = -radius; dx <= radius + 1; dx++) {
                double d = Math.sqrt(dx * dx + dy * dy);
                boolean in = dx <= radius && d <= outer && d > inner;
                if (in && runStart == Integer.MIN_VALUE) {
                    runStart = dx;
                } else if (!in && runStart != Integer.MIN_VALUE) {
                    ctx.fill(cx + runStart, cy + dy, cx + dx, cy + dy + 1, argb);
                    runStart = Integer.MIN_VALUE;
                }
            }
        }
    }

    private void renderAttackIndicator(DrawContext ctx, int cx, int cy, int argb) {
        if (mc.options.getAttackIndicator().getValue() != AttackIndicator.CROSSHAIR) {
            return;
        }
        float progress = mc.player.getAttackCooldownProgress(0f);
        if (progress >= 1f) {
            return;
        }
        int w = 16;
        int y = cy + gap.getInt() + size.getInt() + 4;
        ctx.fill(cx - w / 2, y, cx + w / 2, y + 2, 0x80000000);
        ctx.fill(cx - w / 2, y, cx - w / 2 + Math.round(w * progress), y + 2, argb);
    }
}
