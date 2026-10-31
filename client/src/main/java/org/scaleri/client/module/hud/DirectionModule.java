package org.scaleri.client.module.hud;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;

/** A compass strip across the top of the screen, like in most shooters. */
public class DirectionModule extends HudModule {
    private static final String[] LABELS = {"N", "NE", "E", "SE", "S", "SW", "W", "NW"};
    private static final int WIDTH = 140;
    /** Degrees visible either side of the center mark. */
    private static final int SPAN = 70;

    private final BoolSetting background = add(new BoolSetting("Background", true));
    private final BoolSetting showDegrees = add(new BoolSetting("Show Degrees", true));

    public DirectionModule() {
        super("Direction HUD", "A compass bar that shows which way you're facing.", false, 0.39, 0.0);
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        float yaw = mc.player == null ? 0 : mc.player.getYaw(tickDelta);
        // Minecraft's yaw is 0 facing south; turn it into a compass bearing with 0 = north.
        float heading = MathHelper.floorMod(yaw + 180f, 360f);

        width = WIDTH;
        height = showDegrees.isOn() ? 24 : 14;
        if (background.isOn()) {
            Theme.roundRect(context, 0, 0, width, 14, 3, 0x6F000000);
        }
        int center = WIDTH / 2;
        double scale = (double) (WIDTH / 2 - 6) / SPAN;
        int first = (int) Math.ceil((heading - SPAN) / 15f) * 15;
        for (int deg = first; deg <= heading + SPAN; deg += 15) {
            int x = center + (int) Math.round((deg - heading) * scale);
            int bearing = Math.floorMod(deg, 360);
            if (bearing % 45 == 0) {
                String label = LABELS[bearing / 45];
                int color = label.length() == 1 ? Theme.accent() : 0xFFFFFFFF;
                context.drawText(mc.textRenderer, label, x - mc.textRenderer.getWidth(label) / 2, 3, color, true);
            } else {
                context.fill(x, 5, x + 1, 9, 0xA0FFFFFF);
            }
        }
        context.fill(center, 0, center + 1, 2, Theme.accent());
        context.fill(center, 12, center + 1, 14, Theme.accent());
        if (showDegrees.isOn()) {
            String text = Math.round(heading) % 360 + "°";
            context.drawText(mc.textRenderer, text, center - mc.textRenderer.getWidth(text) / 2, 16, 0xFFFFFFFF, true);
        }
    }
}
