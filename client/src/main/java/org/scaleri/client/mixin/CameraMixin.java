package org.scaleri.client.mixin;

import net.minecraft.client.render.Camera;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.visual.FreelookModule;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Camera.class)
public abstract class CameraMixin {
    @Shadow
    protected abstract void setRotation(float yaw, float pitch);

    @Redirect(method = "update", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/client/render/Camera;setRotation(FF)V"))
    private void scaleri$freelookRotation(Camera camera, float yaw, float pitch) {
        FreelookModule freelook = ScaleriClient.get().getModules().getFreelook();
        if (freelook.isEnabled() && freelook.isActive()) {
            setRotation(freelook.getYaw(), freelook.getPitch());
        } else {
            setRotation(yaw, pitch);
        }
    }
}
