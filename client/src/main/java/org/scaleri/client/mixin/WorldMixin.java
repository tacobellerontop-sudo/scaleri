package org.scaleri.client.mixin;

import net.minecraft.client.world.ClientWorld;
import net.minecraft.world.World;
import org.scaleri.client.ScaleriClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(World.class)
public class WorldMixin {
    @Inject(method = {"getRainGradient", "getThunderGradient"}, at = @At("HEAD"), cancellable = true)
    private void scaleri$clearWeather(float delta, CallbackInfoReturnable<Float> cir) {
        if ((Object) this instanceof ClientWorld) {
            ScaleriClient client = ScaleriClient.get();
            if (client != null && client.getModules().getClearWeather().isEnabled()) {
                cir.setReturnValue(0f);
            }
        }
    }
}
