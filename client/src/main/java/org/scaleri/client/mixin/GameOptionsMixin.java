package org.scaleri.client.mixin;

import net.minecraft.client.option.GameOptions;
import org.scaleri.client.ScaleriClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Keeps Fullbright's boosted gamma out of options.txt. */
@Mixin(GameOptions.class)
public class GameOptionsMixin {
    @Inject(method = "write", at = @At("HEAD"))
    private void scaleri$restoreGamma(CallbackInfo ci) {
        ScaleriClient client = ScaleriClient.get();
        if (client != null) {
            client.getModules().getFullbright().restore();
        }
    }
}
