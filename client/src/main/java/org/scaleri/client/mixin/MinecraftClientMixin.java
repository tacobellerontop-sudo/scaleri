package org.scaleri.client.mixin;

import net.minecraft.SharedConstants;
import net.minecraft.client.MinecraftClient;
import org.scaleri.client.ScaleriClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MinecraftClient.class)
public class MinecraftClientMixin {
    @Inject(method = "getWindowTitle", at = @At("RETURN"), cancellable = true)
    private void scaleri$windowTitle(CallbackInfoReturnable<String> cir) {
        ScaleriClient client = ScaleriClient.get();
        if (client == null || client.getModules().getTheme().windowTitle.isOn()) {
            cir.setReturnValue(ScaleriClient.NAME + " " + ScaleriClient.version()
                    + " • Minecraft " + SharedConstants.getGameVersion().getName());
        }
    }
}
