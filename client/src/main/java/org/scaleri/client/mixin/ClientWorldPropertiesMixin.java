package org.scaleri.client.mixin;

import net.minecraft.client.world.ClientWorld;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.visual.TimeChangerModule;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(ClientWorld.Properties.class)
public class ClientWorldPropertiesMixin {
    @Inject(method = "getTimeOfDay", at = @At("HEAD"), cancellable = true)
    private void scaleri$timeChanger(CallbackInfoReturnable<Long> cir) {
        ScaleriClient client = ScaleriClient.get();
        if (client == null) {
            return;
        }
        TimeChangerModule time = client.getModules().getTimeChanger();
        if (time.isEnabled()) {
            cir.setReturnValue(time.getTime());
        }
    }
}
