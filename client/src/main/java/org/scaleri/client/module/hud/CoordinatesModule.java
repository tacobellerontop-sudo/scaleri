package org.scaleri.client.module.hud;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;

public class CoordinatesModule extends HudModule {
    private final BoolSetting background = add(new BoolSetting("Background", true));
    private final BoolSetting showDirection = add(new BoolSetting("Show Direction", true));
    private final BoolSetting showBiome = add(new BoolSetting("Show Biome", false));
    private final ColorSetting labelColor = add(new ColorSetting("Label Color", 0xFF4DABF7));
    private final ColorSetting valueColor = add(new ColorSetting("Value Color", 0xFFFFFFFF));

    public CoordinatesModule() {
        super("Coordinates", "Shows where you are and which way you're facing.", true, 0.004, 0.116);
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        List<String[]> lines = new ArrayList<>();
        if (mc.player == null) {
            lines.add(new String[] {"X", "128"});
            lines.add(new String[] {"Y", "64"});
            lines.add(new String[] {"Z", "-256"});
            if (showDirection.isOn()) {
                lines.add(new String[] {"Facing", "North (-Z)"});
            }
            if (showBiome.isOn()) {
                lines.add(new String[] {"Biome", "Plains"});
            }
        } else {
            BlockPos pos = mc.player.getBlockPos();
            lines.add(new String[] {"X", Integer.toString(pos.getX())});
            lines.add(new String[] {"Y", Integer.toString(pos.getY())});
            lines.add(new String[] {"Z", Integer.toString(pos.getZ())});
            if (showDirection.isOn()) {
                lines.add(new String[] {"Facing", describe(mc.player.getHorizontalFacing())});
            }
            if (showBiome.isOn() && mc.world != null) {
                String biome = mc.world.getBiome(pos).getKey().map(k -> prettify(k.getValue().getPath())).orElse("Unknown");
                lines.add(new String[] {"Biome", biome});
            }
        }

        int pad = background.isOn() ? 5 : 0;
        int maxWidth = 0;
        for (String[] line : lines) {
            maxWidth = Math.max(maxWidth, mc.textRenderer.getWidth(line[0] + ": " + line[1]));
        }
        width = maxWidth + pad * 2;
        height = lines.size() * 10 - 1 + pad * 2;
        if (background.isOn()) {
            Theme.roundRect(context, 0, 0, width, height, 3, 0x6F000000);
        }
        int y = pad;
        for (String[] line : lines) {
            int x = context.drawText(mc.textRenderer, line[0] + ": ", pad, y, labelColor.getArgb(), true);
            context.drawText(mc.textRenderer, line[1], x - 1, y, valueColor.getArgb(), true);
            y += 10;
        }
    }

    private static String describe(Direction facing) {
        return switch (facing) {
            case NORTH -> "North (-Z)";
            case SOUTH -> "South (+Z)";
            case EAST -> "East (+X)";
            case WEST -> "West (-X)";
            default -> facing.asString();
        };
    }

    private static String prettify(String id) {
        StringBuilder sb = new StringBuilder();
        for (String word : id.split("_")) {
            if (!word.isEmpty()) {
                sb.append(Character.toUpperCase(word.charAt(0))).append(word.substring(1)).append(' ');
            }
        }
        return sb.toString().trim();
    }
}
