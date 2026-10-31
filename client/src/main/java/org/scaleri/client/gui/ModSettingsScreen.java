package org.scaleri.client.gui;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;
import org.scaleri.client.setting.ModeSetting;
import org.scaleri.client.setting.NumberSetting;
import org.scaleri.client.setting.Setting;

/** Lists and edits one module's settings. */
public class ModSettingsScreen extends ScaleriScreen {
    private static final int ROW = 24;
    private static final int COLOR_ROW = 50;
    private static final int SLIDER_W = 120;

    private final Module module;
    private double scroll;
    private double maxScroll;

    public ModSettingsScreen(Screen parent, Module module) {
        super(module.getName(), parent);
        this.module = module;
    }

    @Override
    protected void draw(DrawContext ctx, int mouseX, int mouseY, float delta) {
        int pw = Math.min(width - 24, 340);
        int ph = Math.min(height - 24, 320);
        int px = (width - pw) / 2;
        int py = (height - ph) / 2;
        Theme.roundRect(ctx, px, py, pw, ph, 6, Theme.PANEL);

        // Header
        boolean backHover = Theme.inside(mouseX, mouseY, px + 8, py + 8, 40, 16);
        ctx.drawText(textRenderer, "‹ Back", px + 10, py + 12, backHover ? Theme.TEXT : Theme.TEXT_DIM, false);
        onClick(px + 8, py + 8, 40, 16, (mx, my, b) -> close());

        String title = module.getName();
        ctx.drawText(textRenderer, title, px + (pw - textRenderer.getWidth(title)) / 2, py + 12, Theme.TEXT, true);
        if (module.isToggleable()) {
            Theme.toggle(ctx, px + pw - 30, py + 11, module.isEnabled() ? 1f : 0f);
            onClick(px + pw - 32, py + 9, 24, 14, (mx, my, b) -> module.toggle());
        }
        ctx.fill(px + 8, py + 30, px + pw - 8, py + 31, 0x20FFFFFF);

        // Settings list
        int top = py + 36;
        int bottom = py + ph - 34;
        int x = px + 12;
        int w = pw - 24;
        int contentHeight = 0;
        for (Setting<?> s : module.getSettings()) {
            contentHeight += s instanceof ColorSetting ? COLOR_ROW : ROW;
        }
        maxScroll = Math.max(0, contentHeight - (bottom - top));
        scroll = MathHelper.clamp(scroll, 0, maxScroll);

        ctx.enableScissor(px, top, px + pw, bottom);
        clip(top, bottom);
        int y = top - (int) scroll;
        for (Setting<?> s : module.getSettings()) {
            int h = s instanceof ColorSetting ? COLOR_ROW : ROW;
            if (y + h >= top && y <= bottom) {
                row(ctx, s, x, y, w, mouseX, mouseY);
            }
            y += h;
        }
        unclip();
        ctx.disableScissor();

        // Footer
        int by = py + ph - 26;
        button(ctx, px + 8, by, 90, 18, "Reset", false, mouseX, mouseY, () -> module.getSettings().forEach(Setting::reset));
        if (module instanceof HudModule) {
            button(ctx, px + pw - 98, by, 90, 18, "Move on HUD", true, mouseX, mouseY,
                    () -> client.setScreen(new HudEditorScreen(this)));
        }
    }

    private void row(DrawContext ctx, Setting<?> setting, int x, int y, int w, int mouseX, int mouseY) {
        ctx.drawText(textRenderer, setting.getName(), x, y + 8, Theme.TEXT, false);
        int right = x + w;
        if (setting instanceof BoolSetting bool) {
            Theme.toggle(ctx, right - 20, y + 7, bool.isOn() ? 1f : 0f);
            onClick(x, y, w, ROW, (mx, my, b) -> bool.toggle());
        } else if (setting instanceof NumberSetting num) {
            String value = num.format();
            int sx = right - SLIDER_W;
            ctx.drawText(textRenderer, value, sx - textRenderer.getWidth(value) - 6, y + 8, Theme.TEXT_DIM, false);
            slider(ctx, sx, y + 10, SLIDER_W, num.getFraction(), mouseX, mouseY,
                    (mx, my) -> num.setFraction((mx - sx) / SLIDER_W));
        } else if (setting instanceof ModeSetting mode) {
            int bw = 100;
            int bx = right - bw;
            Theme.roundRect(ctx, bx, y + 3, bw, 18, 4, Theme.CARD);
            String v = textRenderer.trimToWidth(mode.get(), bw - 24);
            ctx.drawText(textRenderer, v, bx + (bw - textRenderer.getWidth(v)) / 2, y + 8, Theme.TEXT, false);
            boolean leftHover = Theme.inside(mouseX, mouseY, bx, y + 3, bw / 2, 18);
            ctx.drawText(textRenderer, "‹", bx + 5, y + 8, leftHover ? Theme.accent() : Theme.TEXT_FAINT, false);
            ctx.drawText(textRenderer, "›", bx + bw - 9, y + 8, !leftHover && Theme.inside(mouseX, mouseY, bx, y + 3, bw, 18)
                    ? Theme.accent() : Theme.TEXT_FAINT, false);
            onClick(bx, y + 3, bw, 18, (mx, my, b) -> mode.cycle(mx >= bx + bw / 2.0 && b == 0));
        } else if (setting instanceof ColorSetting color) {
            colorRow(ctx, color, x, y, w, mouseX, mouseY);
        }
    }

    private void colorRow(DrawContext ctx, ColorSetting color, int x, int y, int w, int mouseX, int mouseY) {
        int right = x + w;
        // Current color preview over a checkerboard so transparency shows.
        int pvx = right - 18;
        for (int i = 0; i < 4; i++) {
            ctx.fill(pvx + (i % 2) * 9, y + 4 + (i / 2) * 6, pvx + (i % 2) * 9 + 9, y + 10 + (i / 2) * 6,
                    (i == 0 || i == 3) ? 0xFFCCCCCC : 0xFF888888);
        }
        ctx.fill(pvx, y + 4, pvx + 18, y + 16, color.getArgb());
        Theme.outline(ctx, pvx - 1, y + 3, 20, 14, 0x40FFFFFF);

        int sy = y + 22;
        int sx = x;
        for (int rgb : ColorSetting.PALETTE) {
            boolean selected = !color.isChroma() && (color.get() & 0xFFFFFF) == (rgb & 0xFFFFFF);
            if (selected) {
                ctx.fill(sx - 1, sy - 1, sx + 11, sy + 11, Theme.accent());
            }
            ctx.fill(sx, sy, sx + 10, sy + 10, rgb | 0xFF000000);
            int swatch = rgb;
            onClick(sx, sy, 10, 10, (mx, my, b) -> {
                color.setChroma(false);
                color.setRgb(swatch);
            });
            sx += 12;
        }
        // Chroma pill
        int cx = sx + 4;
        int cw = textRenderer.getWidth("Chroma") + 10;
        Theme.roundRect(ctx, cx, sy - 2, cw, 14, 4, color.isChroma() ? Theme.accent() : Theme.CARD);
        ctx.drawText(textRenderer, "Chroma", cx + 5, sy + 1, Theme.TEXT, false);
        onClick(cx, sy - 2, cw, 14, (mx, my, b) -> color.setChroma(!color.isChroma()));

        // Opacity
        int oy = y + 38;
        ctx.drawText(textRenderer, "Opacity", x, oy - 3, Theme.TEXT_DIM, false);
        int ox = right - SLIDER_W;
        String pct = Math.round(color.getAlpha() / 2.55f) + "%";
        ctx.drawText(textRenderer, pct, ox - textRenderer.getWidth(pct) - 6, oy - 3, Theme.TEXT_DIM, false);
        slider(ctx, ox, oy, SLIDER_W, color.getAlpha() / 255.0, mouseX, mouseY,
                (mx, my) -> color.setAlpha((int) Math.round(MathHelper.clamp((mx - ox) / SLIDER_W, 0, 1) * 255)));
    }

    private void slider(DrawContext ctx, int x, int y, int w, double fraction, int mouseX, int mouseY, DragAction onChange) {
        int filled = (int) Math.round(w * MathHelper.clamp(fraction, 0, 1));
        Theme.roundRect(ctx, x, y - 1, w, 4, 2, Theme.OFF);
        Theme.roundRect(ctx, x, y - 1, Math.max(4, filled), 4, 2, Theme.accent());
        boolean hover = Theme.inside(mouseX, mouseY, x - 4, y - 6, w + 8, 14);
        int knob = hover ? 8 : 6;
        Theme.roundRect(ctx, x + filled - knob / 2, y + 1 - knob / 2, knob, knob, knob / 2, 0xFFFFFFFF);
        onDrag(x - 4, y - 6, w + 8, 14, onChange);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontalAmount, double verticalAmount) {
        scroll = MathHelper.clamp(scroll - verticalAmount * 20, 0, maxScroll);
        return true;
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
