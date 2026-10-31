package org.scaleri.client.mixin;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.pvp.CrosshairModule;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class InGameHudMixin {
    @Inject(method = "renderCrosshair", at = @At("HEAD"), cancellable = true)
    private void scaleri$crosshair(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        CrosshairModule crosshair = ScaleriClient.get().getModules().getCrosshair();
        if (crosshair.isEnabled()) {
            crosshair.renderCrosshair(context);
            ci.cancel();
        }
    }

    @Inject(method = "renderStatusEffectOverlay", at = @At("HEAD"), cancellable = true)
    private void scaleri$hideEffects(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        if (ScaleriClient.get().getModules().getPotionEffects().shouldHideVanilla()) {
            ci.cancel();
        }
    }
}
