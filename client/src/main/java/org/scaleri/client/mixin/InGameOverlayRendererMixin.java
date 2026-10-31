package org.scaleri.client.mixin;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.hud.InGameOverlayRenderer;
import net.minecraft.client.util.math.MatrixStack;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.pvp.LowFireModule;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameOverlayRenderer.class)
public class InGameOverlayRendererMixin {
    @Inject(method = "renderFireOverlay", at = @At("HEAD"))
    private static void scaleri$lowerFire(MinecraftClient client, MatrixStack matrices, CallbackInfo ci) {
        LowFireModule lowFire = ScaleriClient.get().getModules().getLowFire();
        matrices.push();
        if (lowFire.isEnabled()) {
            matrices.translate(0, -lowFire.offset.get(), 0);
        }
    }

    @Inject(method = "renderFireOverlay", at = @At("TAIL"))
    private static void scaleri$lowerFireEnd(MinecraftClient client, MatrixStack matrices, CallbackInfo ci) {
        matrices.pop();
    }
}
