package org.scaleri.client.gui;

import net.minecraft.client.gui.DrawContext;
import org.scaleri.client.ScaleriClient;

/** Colors and drawing helpers shared by all Scaleri screens. */
public final class Theme {
    public static final int OVERLAY = 0x88000000;
    public static final int PANEL = 0xF0181A20;
    public static final int SIDEBAR = 0xFF14161B;
    public static final int CARD = 0xFF22252D;
    public static final int CARD_HOVER = 0xFF2B2F39;
    public static final int FIELD = 0xFF101216;
    public static final int TEXT = 0xFFF1F3F7;
    public static final int TEXT_DIM = 0xFF9AA0AC;
    public static final int TEXT_FAINT = 0xFF636976;
    public static final int OFF = 0xFF3A3E48;
    public static final int DANGER = 0xFFFF6B6B;

    private Theme() {
    }

    /** The accent color picked in the Theme module. */
    public static int accent() {
        return ScaleriClient.get().getModules().getTheme().accent.getArgb() | 0xFF000000;
    }

    public static int withAlpha(int argb, int alpha) {
        return (alpha & 0xFF) << 24 | (argb & 0xFFFFFF);
    }

    /** Mixes two colors; t = 0 gives a, t = 1 gives b. */
    public static int blend(int a, int b, float t) {
        int r = 0;
        for (int shift = 0; shift <= 24; shift += 8) {
            int ca = a >>> shift & 0xFF;
            int cb = b >>> shift & 0xFF;
            r |= Math.round(ca + (cb - ca) * t) << shift;
        }
        return r;
    }

    /** A filled rectangle with softly rounded corners. */
    public static void roundRect(DrawContext ctx, int x, int y, int w, int h, int radius, int color) {
        if ((color >>> 24) == 0 || w <= 0 || h <= 0) {
            return;
        }
        int r = Math.min(radius, Math.min(w, h) / 2);
        ctx.fill(x, y + r, x + w, y + h - r, color);
        for (int i = 0; i < r; i++) {
            double dy = r - i - 0.5;
            int inset = (int) Math.round(r - Math.sqrt(r * r - dy * dy));
            ctx.fill(x + inset, y + i, x + w - inset, y + i + 1, color);
            ctx.fill(x + inset, y + h - i - 1, x + w - inset, y + h - i, color);
        }
    }

    /** A one pixel outline. */
    public static void outline(DrawContext ctx, int x, int y, int w, int h, int color) {
        ctx.fill(x, y, x + w, y + 1, color);
        ctx.fill(x, y + h - 1, x + w, y + h, color);
        ctx.fill(x, y + 1, x + 1, y + h - 1, color);
        ctx.fill(x + w - 1, y + 1, x + w, y + h - 1, color);
    }

    /** An iOS-style on/off switch, 20x10. {@code progress} animates from 0 (off) to 1 (on). */
    public static void toggle(DrawContext ctx, int x, int y, float progress) {
        int track = blend(OFF, accent(), progress);
        roundRect(ctx, x, y, 20, 10, 5, track);
        int knobX = x + 1 + Math.round(10 * progress);
        roundRect(ctx, knobX, y + 1, 8, 8, 4, 0xFFFFFFFF);
    }

    public static boolean inside(double mx, double my, int x, int y, int w, int h) {
        return mx >= x && my >= y && mx < x + w && my < y + h;
    }
}
