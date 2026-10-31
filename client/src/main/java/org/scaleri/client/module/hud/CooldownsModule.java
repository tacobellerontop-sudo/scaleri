package org.scaleri.client.module.hud;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.item.Item;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;

/** Shows how long until you can use your pearl, chorus fruit, shield or wind charge again. */
public class CooldownsModule extends HudModule {
    private static final List<ItemStack> TRACKED = List.of(
            new ItemStack(Items.ENDER_PEARL),
            new ItemStack(Items.CHORUS_FRUIT),
            new ItemStack(Items.SHIELD),
            new ItemStack(Items.WIND_CHARGE),
            new ItemStack(Items.GOAT_HORN));
    private static final int BAR = 48;

    private final BoolSetting background = add(new BoolSetting("Background", true));

    public CooldownsModule() {
        super("Item Cooldowns", "Shows when your ender pearl, shield and other items are ready again.", true, 0.46, 0.62);
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        List<ItemStack> active = new ArrayList<>();
        List<Float> progress = new ArrayList<>();
        if (mc.player != null) {
            for (ItemStack stack : TRACKED) {
                Item item = stack.getItem();
                float left = mc.player.getItemCooldownManager().getCooldownProgress(item, tickDelta);
                if (left > 0) {
                    active.add(stack);
                    progress.add(left);
                }
            }
        }
        if (active.isEmpty() && editing) {
            active.add(TRACKED.get(0));
            progress.add(0.6f);
            active.add(TRACKED.get(2));
            progress.add(0.25f);
        }
        width = 18 + BAR + 8;
        if (active.isEmpty()) {
            height = 18;
            return;
        }
        height = active.size() * 18 + 2;
        if (background.isOn()) {
            Theme.roundRect(context, 0, 0, width, height, 3, 0x6F000000);
        }
        for (int i = 0; i < active.size(); i++) {
            int y = 1 + i * 18;
            context.drawItem(active.get(i), 2, y);
            int bx = 21;
            int by = y + 7;
            float left = progress.get(i);
            context.fill(bx, by, bx + BAR, by + 3, 0x80000000);
            context.fill(bx, by, bx + Math.round(BAR * (1 - left)), by + 3,
                    Theme.blend(Theme.DANGER, Theme.accent(), 1 - left));
        }
    }
}
