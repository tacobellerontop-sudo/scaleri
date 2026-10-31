package org.scaleri.client.module.misc;

import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import net.minecraft.util.math.BlockPos;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;

public class DeathCoordsModule extends Module {
    private boolean announced;

    public DeathCoordsModule() {
        super("Death Coordinates", "Tells you in chat where you died so you can get your stuff back.", Category.MISC, true);
    }

    @Override
    public void onTick() {
        if (mc.player == null) {
            announced = false;
            return;
        }
        if (mc.player.isDead()) {
            if (!announced) {
                announced = true;
                BlockPos pos = mc.player.getBlockPos();
                String dimension = mc.player.getWorld().getRegistryKey().getValue().getPath().replace('_', ' ');
                mc.player.sendMessage(Text.literal("You died at ").formatted(Formatting.GRAY)
                        .append(Text.literal(pos.getX() + ", " + pos.getY() + ", " + pos.getZ()).formatted(Formatting.WHITE))
                        .append(Text.literal(" in the " + dimension + ".").formatted(Formatting.GRAY)), false);
            }
        } else {
            announced = false;
        }
    }
}
