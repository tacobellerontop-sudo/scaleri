package org.scaleri.client.module.hud;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.texture.Sprite;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.entity.effect.StatusEffects;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.HudModule;
import org.scaleri.client.setting.BoolSetting;

public class PotionEffectsModule extends HudModule {
    private static final String[] ROMAN = {"I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"};

    private final BoolSetting background = add(new BoolSetting("Background", true));
    private final BoolSetting hideVanilla = add(new BoolSetting("Hide Vanilla Icons", true));
    private final BoolSetting blinkWhenEnding = add(new BoolSetting("Blink When Ending", true));

    public PotionEffectsModule() {
        super("Potion Effects", "Lists your active effects and how long they last.", true, 1.0, 0.56);
    }

    public boolean shouldHideVanilla() {
        return isEnabled() && hideVanilla.isOn();
    }

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        List<StatusEffectInstance> effects = new ArrayList<>();
        if (mc.player != null) {
            effects.addAll(mc.player.getStatusEffects());
        }
        if (effects.isEmpty() && editing) {
            effects.add(new StatusEffectInstance(StatusEffects.SPEED, 20 * 90, 1));
            effects.add(new StatusEffectInstance(StatusEffects.STRENGTH, 20 * 8, 0));
        }
        if (effects.isEmpty()) {
            return;
        }

        int rowHeight = 22;
        int maxText = 0;
        for (StatusEffectInstance effect : effects) {
            maxText = Math.max(maxText, Math.max(mc.textRenderer.getWidth(title(effect)), mc.textRenderer.getWidth(duration(effect))));
        }
        width = 18 + 4 + maxText + 8;
        height = effects.size() * rowHeight;

        int y = 0;
        for (StatusEffectInstance effect : effects) {
            if (background.isOn()) {
                Theme.roundRect(context, 0, y, width, rowHeight - 2, 3, 0x6F000000);
            }
            boolean ending = !effect.isInfinite() && effect.getDuration() < 200;
            boolean visible = !blinkWhenEnding.isOn() || !ending || (System.currentTimeMillis() / 400) % 2 == 0;
            if (visible) {
                Sprite sprite = mc.getStatusEffectSpriteManager().getSprite(effect.getEffectType());
                context.drawSprite(2, y + 1, 0, 18, 18, sprite);
            }
            int color = effect.getEffectType().value().getColor() | 0xFF000000;
            context.drawText(mc.textRenderer, title(effect), 24, y + 1, Theme.blend(color, 0xFFFFFFFF, 0.45f), true);
            context.drawText(mc.textRenderer, duration(effect), 24, y + 11, ending ? Theme.DANGER : Theme.TEXT_DIM, true);
            y += rowHeight;
        }
    }

    private static String title(StatusEffectInstance effect) {
        String name = effect.getEffectType().value().getName().getString();
        int amp = effect.getAmplifier();
        return amp == 0 ? name : name + " " + (amp < ROMAN.length ? ROMAN[amp] : Integer.toString(amp + 1));
    }

    private static String duration(StatusEffectInstance effect) {
        if (effect.isInfinite()) {
            return "**:**";
        }
        int seconds = effect.getDuration() / 20;
        return String.format("%d:%02d", seconds / 60, seconds % 60);
    }
}
