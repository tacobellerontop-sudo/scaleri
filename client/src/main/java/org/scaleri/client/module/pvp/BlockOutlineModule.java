package org.scaleri.client.module.pvp;

import net.fabricmc.fabric.api.client.rendering.v1.WorldRenderContext;
import net.minecraft.block.BlockState;
import net.minecraft.block.ShapeContext;
import net.minecraft.client.render.VertexConsumer;
import net.minecraft.client.render.debug.DebugRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Box;
import net.minecraft.util.shape.VoxelShape;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;

public class BlockOutlineModule extends Module {
    private final ColorSetting outlineColor = add(new ColorSetting("Outline Color", 0xCC4DABF7));
    private final BoolSetting fill = add(new BoolSetting("Fill", true));
    private final ColorSetting fillColor = add(new ColorSetting("Fill Color", 0x284DABF7));

    public BlockOutlineModule() {
        super("Block Outline", "Recolors the outline around the block you're looking at.", Category.PVP, false);
    }

    /** Draws the outline; returns false so vanilla skips its own. */
    public boolean render(WorldRenderContext context, VertexConsumer lines, BlockPos pos, BlockState state,
                          double camX, double camY, double camZ) {
        MatrixStack matrices = context.matrixStack();
        if (matrices == null || mc.world == null || mc.player == null) {
            return true;
        }
        VoxelShape shape = state.getOutlineShape(mc.world, pos, ShapeContext.of(mc.player));
        if (shape.isEmpty()) {
            return true;
        }
        double ox = pos.getX() - camX;
        double oy = pos.getY() - camY;
        double oz = pos.getZ() - camZ;

        int c = outlineColor.getArgb();
        int r = c >> 16 & 0xFF;
        int g = c >> 8 & 0xFF;
        int b = c & 0xFF;
        int a = c >>> 24;
        MatrixStack.Entry entry = matrices.peek();
        shape.forEachEdge((x1, y1, z1, x2, y2, z2) -> {
            float nx = (float) (x2 - x1);
            float ny = (float) (y2 - y1);
            float nz = (float) (z2 - z1);
            float len = (float) Math.sqrt(nx * nx + ny * ny + nz * nz);
            nx /= len;
            ny /= len;
            nz /= len;
            lines.vertex(entry, (float) (x1 + ox), (float) (y1 + oy), (float) (z1 + oz)).color(r, g, b, a).normal(entry, nx, ny, nz);
            lines.vertex(entry, (float) (x2 + ox), (float) (y2 + oy), (float) (z2 + oz)).color(r, g, b, a).normal(entry, nx, ny, nz);
        });

        // Fill after the outline: asking for another buffer can flush the lines buffer.
        if (fill.isOn() && context.consumers() != null) {
            int fc = fillColor.getArgb();
            float fr = (fc >> 16 & 0xFF) / 255f;
            float fg = (fc >> 8 & 0xFF) / 255f;
            float fb = (fc & 0xFF) / 255f;
            float fa = (fc >>> 24) / 255f;
            for (Box box : shape.getBoundingBoxes()) {
                DebugRenderer.drawBox(matrices, context.consumers(), box.offset(ox, oy, oz).expand(0.002), fr, fg, fb, fa);
            }
        }
        return false;
    }
}
