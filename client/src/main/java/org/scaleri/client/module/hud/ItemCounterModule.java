package org.scaleri.client.module.hud;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Predicate;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.component.DataComponentTypes;
import net.minecraft.component.type.PotionContentsComponent;
import net.minecraft.entity.player.PlayerInventory;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.potion.Potions;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ModeSetting;

/** Counts the PvP items left in your inventory: pots, gapples, pearls, arrows and totems. */
public class ItemCounterModule extends HudModule {
    private final ModeSetting layout = add(new ModeSetting("Layout", "Vertical", "Vertical", "Horizontal"));
    private final BoolSetting background = add(new BoolSetting("Background", true));
    private final BoolSetting hideEmpty = add(new BoolSetting("Hide Items You Don't Have", true));
    private final BoolSetting potions = add(new BoolSetting("Healing Potions", true));
    private final BoolSetting gapples = add(new BoolSetting("Golden Apples", true));
    private final BoolSetting pearls = add(new BoolSetting("Ender Pearls", true));
    private final BoolSetting arrows = add(new BoolSetting("Arrows", true));
    private final BoolSetting totems = add(new BoolSetting("Totems", false));

    private final List<Entry> entries = List.of(
            new Entry(potions, new ItemStack(Items.SPLASH_POTION), ItemCounterModule::isHealingSplash, 14),
            new Entry(gapples, new ItemStack(Items.GOLDEN_APPLE),
                    s -> s.isOf(Items.GOLDEN_APPLE) || s.isOf(Items.ENCHANTED_GOLDEN_APPLE), 6),
            new Entry(pearls, new ItemStack(Items.ENDER_PEARL), s -> s.isOf(Items.ENDER_PEARL), 4),
            new Entry(arrows, new ItemStack(Items.ARROW),
                    s -> s.isOf(Items.ARROW) || s.isOf(Items.SPECTRAL_ARROW) || s.isOf(Items.TIPPED_ARROW), 32),
            new Entry(totems, new ItemStack(Items.TOTEM_OF_UNDYING), s -> s.isOf(Items.TOTEM_OF_UNDYING), 1));

    public ItemCounterModule() {
        super("Item Counter", "Counts your pots, golden apples, pearls, arrows and totems.", false, 1.0, 0.72);
        entries.get(0).icon().set(DataComponentTypes.POTION_CONTENTS, new PotionContentsComponent(Potions.HEALING));
    }

    private static boolean isHealingSplash(ItemStack stack) {
        if (!stack.isOf(Items.SPLASH_POTION)) {
            return false;
        }
        PotionContentsComponent contents = stack.get(DataComponentTypes.POTION_CONTENTS);
        return contents != null && (contents.matches(Potions.HEALING) || contents.matches(Potions.STRONG_HEALING));
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        List<Entry> shown = new ArrayList<>();
        List<Integer> counts = new ArrayList<>();
        for (Entry entry : entries) {
            if (!entry.enabled.isOn()) {
                continue;
            }
            int count = mc.player == null ? entry.sample : count(mc.player.getInventory(), entry.matcher);
            if (count == 0 && hideEmpty.isOn() && !editing) {
                continue;
            }
            shown.add(entry);
            counts.add(count);
        }
        if (shown.isEmpty()) {
            width = 40;
            height = 16;
            if (editing) {
                context.drawText(mc.textRenderer, getName(), 0, 4, Theme.TEXT_DIM, true);
            }
            return;
        }

        boolean vertical = layout.is("Vertical");
        int pad = background.isOn() ? 3 : 0;
        int cellW = 18 + mc.textRenderer.getWidth("999") + 2;
        int rowH = 18;
        width = (vertical ? cellW : cellW * shown.size()) + pad * 2;
        height = (vertical ? rowH * shown.size() : rowH) + pad * 2 - 2;
        if (background.isOn()) {
            Theme.roundRect(context, 0, 0, width, height, 3, 0x6F000000);
        }
        for (int i = 0; i < shown.size(); i++) {
            int x = pad + (vertical ? 0 : i * cellW);
            int y = pad + (vertical ? i * rowH : 0);
            int count = counts.get(i);
            context.drawItem(shown.get(i).icon, x, y);
            context.drawText(mc.textRenderer, Integer.toString(count), x + 19, y + 4,
                    count == 0 ? Theme.DANGER : 0xFFFFFFFF, true);
        }
    }

    private static int count(PlayerInventory inventory, Predicate<ItemStack> matcher) {
        int total = 0;
        for (int i = 0; i < inventory.size(); i++) {
            ItemStack stack = inventory.getStack(i);
            if (!stack.isEmpty() && matcher.test(stack)) {
                total += stack.getCount();
            }
        }
        return total;
    }

    private record Entry(BoolSetting enabled, ItemStack icon, Predicate<ItemStack> matcher, int sample) {
    }
}
