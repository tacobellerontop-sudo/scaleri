package org.scaleri.client.setting;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.awt.Color;

/** An ARGB color, optionally cycling through the rainbow ("chroma"). */
public class ColorSetting extends Setting<Integer> {
    /** Swatches offered in the settings screen. */
    public static final int[] PALETTE = {
        0xFFFFFFFF, 0xFFB0B4BE, 0xFF4A4E58, 0xFF000000,
        0xFFFF6B6B, 0xFFFFA94D, 0xFFFFD43B, 0xFF69DB7C,
        0xFF38D9A9, 0xFF4DABF7, 0xFF748FFC, 0xFFB197FC,
        0xFFF783AC,
    };

    private boolean chroma;

    public ColorSetting(String name, int defaultArgb) {
        super(name, defaultArgb);
    }

    public boolean isChroma() {
        return chroma;
    }

    public void setChroma(boolean chroma) {
        this.chroma = chroma;
    }

    /** The color to draw with right now, with chroma applied. */
    public int getArgb() {
        if (!chroma) {
            return value;
        }
        float hue = (System.currentTimeMillis() % 4000L) / 4000f;
        return (value & 0xFF000000) | (Color.HSBtoRGB(hue, 0.55f, 1f) & 0xFFFFFF);
    }

    public int getAlpha() {
        return value >>> 24;
    }

    public void setAlpha(int alpha) {
        set((alpha & 0xFF) << 24 | (value & 0xFFFFFF));
    }

    /** Picks a palette swatch while keeping the current alpha. */
    public void setRgb(int rgb) {
        set((value & 0xFF000000) | (rgb & 0xFFFFFF));
    }

    @Override
    public void reset() {
        super.reset();
        chroma = false;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("argb", String.format("%08X", value));
        o.addProperty("chroma", chroma);
        return o;
    }

    @Override
    public void fromJson(JsonElement json) {
        if (!json.isJsonObject()) {
            return;
        }
        JsonObject o = json.getAsJsonObject();
        try {
            if (o.has("argb")) {
                set((int) Long.parseLong(o.get("argb").getAsString(), 16));
            }
        } catch (NumberFormatException ignored) {
            // keep the default
        }
        if (o.has("chroma")) {
            chroma = o.get("chroma").getAsBoolean();
        }
    }
}
