package org.scaleri.client.mixin;

import net.minecraft.client.render.Camera;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.util.math.MatrixStack;
import org.scaleri.client.ScaleriClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(GameRenderer.class)
public class GameRendererMixin {
    @Inject(method = "getFov", at = @At("RETURN"), cancellable = true)
    private void scaleri$zoom(Camera camera, float tickDelta, boolean changingFov, CallbackInfoReturnable<Double> cir) {
        if (!changingFov) {
            return;
        }
        double divisor = ScaleriClient.get().getModules().getZoom().getDivisor();
        if (divisor != 1) {
            cir.setReturnValue(cir.getReturnValue() / divisor);
        }
    }

    @Inject(method = "tiltViewWhenHurt", at = @At("HEAD"), cancellable = true)
    private void scaleri$noHurtCam(MatrixStack matrices, float tickDelta, CallbackInfo ci) {
        if (ScaleriClient.get().getModules().getNoHurtCam().isEnabled()) {
            ci.cancel();
        }
    }
}
