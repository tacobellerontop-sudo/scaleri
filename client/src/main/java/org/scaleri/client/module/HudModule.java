package org.scaleri.client.module;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.setting.NumberSetting;

/**
 * A module that draws something on the in-game HUD. Its position is stored as a
 * fraction of the screen so layouts survive window resizes and GUI scale changes.
 */
public abstract class HudModule extends Module {
    private double x;
    private double y;
    protected final NumberSetting scale = add(new NumberSetting("Scale", 1.0, 0.5, 2.5, 0.05));

    /** Size of the last frame, in unscaled pixels. */
    protected int width = 40;
    protected int height = 10;

    protected HudModule(String name, String description, boolean enabledByDefault, double x, double y) {
        this(name, description, Category.HUD, enabledByDefault, x, y);
    }

    protected HudModule(String name, String description, Category category, boolean enabledByDefault, double x, double y) {
        super(name, description, category, enabledByDefault);
        this.x = x;
        this.y = y;
    }

    /**
     * Draws the module with its top-left corner at (0, 0) and updates {@link #width}
     * and {@link #height}. {@code editing} is true inside the HUD editor, where the
     * module should show sample data when there is nothing real to show.
     */
    public abstract void render(DrawContext context, float tickDelta, boolean editing);

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public void setPosition(double x, double y) {
        this.x = MathHelper.clamp(x, 0, 1);
        this.y = MathHelper.clamp(y, 0, 1);
    }

    public float getScale() {
        return scale.getFloat();
    }

    public void setScale(double value) {
        scale.set(value);
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public int getScaledWidth() {
        return Math.round(width * getScale());
    }

    public int getScaledHeight() {
        return Math.round(height * getScale());
    }

    /** Left edge in screen pixels, kept on screen. */
    public int getScreenX(int screenWidth) {
        int px = (int) Math.round(x * screenWidth);
        return MathHelper.clamp(px, 0, Math.max(0, screenWidth - getScaledWidth()));
    }

    /** Top edge in screen pixels, kept on screen. */
    public int getScreenY(int screenHeight) {
        int py = (int) Math.round(y * screenHeight);
        return MathHelper.clamp(py, 0, Math.max(0, screenHeight - getScaledHeight()));
    }

    /** Draws the module at its saved position and scale. */
    public void renderPlaced(DrawContext context, float tickDelta, boolean editing) {
        int sw = context.getScaledWindowWidth();
        int sh = context.getScaledWindowHeight();
        context.getMatrices().push();
        context.getMatrices().translate(getScreenX(sw), getScreenY(sh), 0);
        context.getMatrices().scale(getScale(), getScale(), 1);
        render(context, tickDelta, editing);
        context.getMatrices().pop();
    }
}
