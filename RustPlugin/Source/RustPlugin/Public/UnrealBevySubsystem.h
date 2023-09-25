#pragma once

#include "CoreMinimal.h"

#include "bevy_capi.h"
#include "UnrealBevySubsystem.generated.h"

/**
 * 
 */
UCLASS()
class RUSTPLUGIN_API UUnrealBevySubsystem : public UGameInstanceSubsystem
{
	GENERATED_BODY()
public:
	virtual void Initialize(FSubsystemCollectionBase& Collection) override;
	virtual void Deinitialize() override;

	bevy_capi::bevy_world* GetEcsWorld() const;
protected:
	FTickerDelegate OnTickDelegate;
	FDelegateHandle OnTickHandle;

	bevy_capi::bevy_world* ECSWorld = nullptr;
	
private:
	bool Tick(float DeltaTime);
};