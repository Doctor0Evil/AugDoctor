#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "CN_XRTypes.h"
#include "CN_XRDataRetriever.generated.h"

UCLASS(ClassGroup=(CyberNano), meta=(BlueprintSpawnableComponent))
class CYBERNANO_API UCN_XRDataRetriever : public UActorComponent
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category="CyberNano|XR")
    bool PullXRHostSnapshot(const FString& HostID, FXRHostSnapshot& OutSnapshot);

    UFUNCTION(BlueprintCallable, Category="CyberNano|XR")
    bool PullXRCorridorEnvelope(const FString& SessionID, FXRSessionEnvelope& OutEnvelope);
};
