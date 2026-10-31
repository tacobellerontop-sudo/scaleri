package org.scaleri.client.module.hud;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ModeSetting;

public class ArmorStatusModule extends HudModule {
    private final ModeSetting layout = add(new ModeSetting("Layout", "Vertical", "Vertical", "Horizontal"));
    private final ModeSetting durability = add(new ModeSetting("Durability", "Value", "Value", "Percent", "Hidden"));
    private final BoolSetting showHeld = add(new BoolSetting("Show Held Item", true));

    public ArmorStatusModule() {
        super("Armor Status", "Shows your armor and its durability.", true, 1.0, 0.31);
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        List<ItemStack> stacks = new ArrayList<>();
        if (mc.player == null) {
            stacks.add(new ItemStack(Items.DIAMOND_HELMET));
            stacks.add(new ItemStack(Items.DIAMOND_CHESTPLATE));
            stacks.add(new ItemStack(Items.IRON_LEGGINGS));
            stacks.add(new ItemStack(Items.DIAMOND_BOOTS));
            if (showHeld.isOn()) {
                stacks.add(new ItemStack(Items.DIAMOND_SWORD));
            }
        } else {
            for (int slot = 3; slot >= 0; slot--) {
                ItemStack stack = mc.player.getInventory().getArmorStack(slot);
                if (!stack.isEmpty()) {
                    stacks.add(stack);
                }
            }
            if (showHeld.isOn() && !mc.player.getMainHandStack().isEmpty()) {
                stacks.add(mc.player.getMainHandStack());
            }
        }
        if (stacks.isEmpty()) {
            width = 16;
            height = 16;
            if (editing) {
                context.drawText(mc.textRenderer, "Armor", 0, 4, Theme.TEXT_DIM, true);
            }
            return;
        }

        boolean vertical = layout.is("Vertical");
        int x = 0;
        int y = 0;
        int maxRight = 0;
        for (ItemStack stack : stacks) {
            context.drawItem(stack, x, y);
            context.drawItemInSlot(mc.textRenderer, stack, x, y);
            String text = durabilityText(stack);
            int textWidth = 0;
            if (text != null) {
                textWidth = mc.textRenderer.getWidth(text) + 2;
                context.drawText(mc.textRenderer, text, x + 18, y + 4, durabilityColor(stack), true);
            }
            maxRight = Math.max(maxRight, x + 16 + textWidth);
            if (vertical) {
                y += 16;
            } else {
                x += 18 + textWidth;
            }
        }
        width = vertical ? maxRight : x - 2;
        height = vertical ? y : 16;
    }

    private String durabilityText(ItemStack stack) {
        if (durability.is("Hidden")) {
            return null;
        }
        if (!stack.isDamageable()) {
            return null;
        }
        int left = stack.getMaxDamage() - stack.getDamage();
        if (durability.is("Percent")) {
            return left * 100 / stack.getMaxDamage() + "%";
        }
        return Integer.toString(left);
    }

    private static int durabilityColor(ItemStack stack) {
        if (!stack.isDamageable()) {
            return 0xFFFFFFFF;
        }
        float left = 1f - (float) stack.getDamage() / stack.getMaxDamage();
        return Theme.blend(0xFFFF6B6B, 0xFF69DB7C, left) | 0xFF000000;
    }
}
