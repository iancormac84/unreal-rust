#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"

class AActor;
class FRustPluginModule;

DECLARE_LOG_CATEGORY_EXTERN(RustVisualLog, Log, All);

Quaternion ToQuaternion(FQuat q);

Vector3 ToVector3(FVector v);

FVector ToFVector(Vector3 v);
FColor ToFColor(Color c);

// W, X, Y, Z
FQuat ToFQuat(Quaternion q);

AActor *ToAActor(const AActorOpaque *actor);
AActor *ToAActor(AActorOpaque *actor);

FGuid ToFGuid(Uuid uuid);
Uuid ToUuid(FGuid guid);

UnrealBindings CreateBindings();
FRustPluginModule& GetModule();

FCollisionShape ToFCollisionShape(CollisionShape Shape);