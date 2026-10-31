package org.scaleri.client.setting;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import net.minecraft.util.math.MathHelper;

public class NumberSetting extends Setting<Double> {
    private final double min;
    private final double max;
    private final double step;

    public NumberSetting(String name, double defaultValue, double min, double max, double step) {
        super(name, defaultValue);
        this.min = min;
        this.max = max;
        this.step = step;
    }

    public double getMin() {
        return min;
    }

    public double getMax() {
        return max;
    }

    public float getFloat() {
        return value.floatValue();
    }

    public int getInt() {
        return (int) Math.round(value);
    }

    @Override
    public void set(Double value) {
        double snapped = step > 0 ? Math.round(value / step) * step : value;
        super.set(MathHelper.clamp(snapped, min, max));
    }

    /** Sets the value from a 0..1 slider position. */
    public void setFraction(double fraction) {
        set(min + (max - min) * MathHelper.clamp(fraction, 0, 1));
    }

    public double getFraction() {
        return (value - min) / (max - min);
    }

    public String format() {
        if (step >= 1) {
            return Integer.toString(getInt());
        }
        return step >= 0.1 ? String.format("%.1f", value) : String.format("%.2f", value);
    }

    @Override
    public JsonElement toJson() {
        return new JsonPrimitive(value);
    }

    @Override
    public void fromJson(JsonElement json) {
        if (json.isJsonPrimitive() && json.getAsJsonPrimitive().isNumber()) {
            set(json.getAsDouble());
        }
    }
}
