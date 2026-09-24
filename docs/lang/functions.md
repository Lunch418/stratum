# Встроенные функции Stratum 2000

Сведено из таблиц компилятора (`template/*.tpl`: типы аргументов и опкоды) и декомпилированной справки `SC3.HLP` (описания).

Всего имён: 968. Из них в обоих источниках: 840, только в таблицах: 119, только в справке: 9.

## Функции работы с Ogre 3D (299)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [AnimationState_AddTime](../help/topics/AnimationState_AddTime.md) | `AnimationState_AddTime(HANDLE, FLOAT)` | 906 | Функция сдвигает текущее состояние анимации на определенный промежуток времени. |
| [AnimationState_GetEnabled](../help/topics/AnimationState_GetEnabled.md) | `FLOAT AnimationState_GetEnabled(HANDLE)` | 907 | Функция используется для определения активности анимации. |
| [AnimationState_GetLength](../help/topics/AnimationState_GetLength.md) | `FLOAT AnimationState_GetLength(HANDLE)` | 902 | Функция получает длину анимации (в секундах). |
| [AnimationState_GetLoop](../help/topics/AnimationState_GetLoop.md) | `FLOAT AnimationState_GetLoop(HANDLE)` | 910 | Функция используется для определения состояния режима зацикливания. |
| [AnimationState_GetTimePosition](../help/topics/AnimationState_GetTimePosition.md) | `FLOAT AnimationState_GetTimePosition(HANDLE)` | 900 | Функция получает текущее состояние анимации во времени. |
| [AnimationState_GetWeight](../help/topics/AnimationState_GetWeight.md) | `FLOAT AnimationState_GetWeight(HANDLE)` | 904 | Функция получает вес анимации при проигрывании. |
| [AnimationState_SetEnabled](../help/topics/AnimationState_SetEnabled.md) | `AnimationState_SetEnabled(HANDLE, FLOAT)` | 908 | Функция устанавливает флаг активности анимации. |
| [AnimationState_SetLength](../help/topics/AnimationState_SetLength.md) | `AnimationState_SetLength(HANDLE, FLOAT)` | 903 | Функция устанавливает длину анимации (в секундах). |
| [AnimationState_SetLoop](../help/topics/AnimationState_SetLoop.md) | `AnimationState_SetLoop(HANDLE, FLOAT)` | 909 | Функция устанавливает режим зацикливания анимации (при завершении анимации она начинает проигрываться сначала). |
| [AnimationState_SetTimePosition](../help/topics/AnimationState_SetTimePosition.md) | `AnimationState_SetTimePosition(HANDLE, FLOAT)` | 901 | Функция устанавливает текущее состояние анимации во времени. |
| [AnimationState_SetWeight](../help/topics/AnimationState_SetWeight.md) | `AnimationState_SetWeight(HANDLE, FLOAT)` | 905 | Функция устанавливает вес анимации. |
| [Billboard_GetColour](../help/topics/Billboard_GetColour.md) | `Billboard_GetColour(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1098 | Функция для получения цвета плоскости. |
| [Billboard_GetPosition](../help/topics/Billboard_GetPosition.md) | `Billboard_GetPosition(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1096 | Функция для полчения координат плоскости. |
| [Billboard_GetRotation](../help/topics/Billboard_GetRotation.md) | `FLOAT Billboard_GetRotation(HANDLE)` | 1094 | Функция для получения угла поворота плоскости. |
| [Billboard_SetColour](../help/topics/Billboard_SetColour.md) | `Billboard_SetColour(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1097 | Функция для установки цвета плоскости. |
| [Billboard_SetPosition](../help/topics/Billboard_SetPosition.md) | `Billboard_SetPosition(HANDLE, FLOAT, FLOAT, FLOAT)` | 1095 | Функция для установки координат плоскости. |
| [Billboard_SetRotation](../help/topics/Billboard_SetRotation.md) | `Billboard_SetRotation(HANDLE, FLOAT)` | 1093 | Функция для установки угла поворота плоскости. |
| [BillboardSet_Create](../help/topics/BillboardSet_Create.md) | `HANDLE BillboardSet_Create(HANDLE, STRING, FLOAT)` | 1077 | Функция для создания объекта класса BillboardSet. |
| [BillboardSet_Destroy](../help/topics/BillboardSet_Destroy.md) | `BillboardSet_Destroy(HANDLE)` | 1078 | Функция для уничтожения объекта класса BillboardSet. |
| [BillboardSet_GetBillboardType](../help/topics/BillboardSet_GetBillboardType.md) | `FLOAT BillboardSet_GetBillboardType(HANDLE)` | 1084 | Определение используемого типа ориентации плоскостей. |
| [BillboardSet_GetCommonUpVector](../help/topics/BillboardSet_GetCommonUpVector.md) | `BillboardSet_GetCommonUpVector(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1088 | Функция для получения вектора up-vector для типов ориентации BBT_PERPENDICULAR_SELF и BBT_PERPENDICULAR_COMMON. |
| [BillboardSet_GetDefaultDimensions](../help/topics/BillboardSet_GetDefaultDimensions.md) | `BillboardSet_GetDefaultDimensions(HANDLE, &FLOAT, &FLOAT)` | 1090 | Функция для получения ширины и высоты плоскостей. |
| [BillboardSet_GetMaterialName](../help/topics/BillboardSet_GetMaterialName.md) | `BillboardSet_GetMaterialName(HANDLE, &STRING)` | 1092 | Функция для получения названия материала плоскостей. |
| [BillboardSet_SetBillboardType](../help/topics/BillboardSet_SetBillboardType.md) | `BillboardSet_SetBillboardType(HANDLE, FLOAT)` | 1083 | Устанавливает тип ориентации плоскостей. |
| [BillboardSet_SetCommonDirection](../help/topics/BillboardSet_GetCommonDirection.md) | `BillboardSet_SetCommonDirection(HANDLE, FLOAT, FLOAT, FLOAT)` | 1085 | Функция для получения вектора направления для типов ориентации BBT_ORIENTED_COMMON и BBT_PERPENDICULAR_COMMON. |
| [BillboardSet_SetCommonUpVector](../help/topics/BillboardSet_SetCommonUpVector.md) | `BillboardSet_SetCommonUpVector(HANDLE, FLOAT, FLOAT, FLOAT)` | 1087 | Функция для установки вектора up-vector для типов ориентации BBT_PERPENDICULAR_SELF и BBT_PERPENDICULAR_COMMON. |
| [BillboardSet_SetDefaultDimensions](../help/topics/BillboardSet_SetDefaultDimensions.md) | `BillboardSet_SetDefaultDimensions(HANDLE, FLOAT, FLOAT)` | 1089 | Функция для установки ширины и высоты плоскостей. |
| [BillboardSet_SetMaterialName](../help/topics/BillboardSet_SetMaterialName.md) | `BillboardSet_SetMaterialName(HANDLE, STRING)` | 1091 | Функция для установки материала плоскостей. |
| [Bone_GetManuallyControlled](../help/topics/Bone_GetManuallyControlled.md) | `FLOAT Bone_GetManuallyControlled(HANDLE)` | 912 | Функция используется для определения состояния режима управления положением кости в пространстве. |
| [Bone_Reset](../help/topics/Bone_Reset.md) | `Bone_Reset(HANDLE)` | 913 | Функция используется для установки начального положения кости в пространстве (начальное положение в скелете трехмерной модели) |
| [Bone_SetManuallyControlled](../help/topics/Bone_SetManuallyControlled.md) | `Bone_SetManuallyControlled(HANDLE, FLOAT)` | 911 | Функция устанавливает режим управления положением кости в пространстве (автоматический или ручной). |
| [Camera_Create](../help/topics/Camera_Create.md) | `HANDLE Camera_Create(HANDLE, STRING)` | 914 | Функция используется для создания камеры. |
| [Camera_Destroy](../help/topics/Camera_Destroy.md) | `Camera_Destroy(HANDLE)` | 915 | Функция удаления камеры. |
| [Camera_GetAspectRatio](../help/topics/Camera_GetAspectRatio.md) | `FLOAT Camera_GetAspectRatio(HANDLE)` | 919 | Функция для получения соотношения сторон плоскости проецирования камеры. |
| [Camera_GetFarClipDistance](../help/topics/Camera_GetFarClipDistance.md) | `FLOAT Camera_GetFarClipDistance(HANDLE)` | 923 | Функция для получения дальней плоскости отсечения камеры |
| [Camera_GetFOV](../help/topics/Camera_GetFOV.md) | `FLOAT Camera_GetFOV(HANDLE)` | 917 | Функция для получения угла зрения камеры. |
| [Camera_GetNearClipDistance](../help/topics/Camera_GetNearClipDistance.md) | `FLOAT Camera_GetNearClipDistance(HANDLE)` | 921 | Функция для получения ближней плоскости отсечения камеры |
| [Camera_GetPolygonMode](../help/topics/Camera_GetPolygonMode.md) | `FLOAT Camera_GetPolygonMode(HANDLE)` | 927 | Функция для получения режима отрисовки полигонов. |
| [Camera_GetProjectionType](../help/topics/Camera_GetProjectionType.md) | `FLOAT Camera_GetProjectionType(HANDLE)` | 925 | Функция для получения типа проецирования. |
| [Camera_SetAspectRatio](../help/topics/Camera_SetAspectRatio.md) | `Camera_SetAspectRatio(HANDLE, FLOAT)` | 918 | Функция устанавливает соотношение сторон плоскости проецирования камеры. |
| [Camera_SetFarClipDistance](../help/topics/Camera_SetFarClipDistance.md) | `Camera_SetFarClipDistance(HANDLE, FLOAT)` | 922 | Функция устанавливает дальнюю плоскость отсечения камеры. |
| [Camera_SetFocalLength](../help/topics/Camera_SetFocalLength.md) | `Camera_SetFocalLength(HANDLE, FLOAT)` | 929 | Функция устанавливает фокусное расстояние. Используется при стерео визуализации. |
| [Camera_SetFOV](../help/topics/Camera_SetFOV.md) | `Camera_SetFOV(HANDLE, FLOAT)` | 916 | Функция устанавливает угол зрения камеры. |
| [Camera_SetFrustumOffset](../help/topics/Camera_SetFrustumOffset.md) | `Camera_SetFrustumOffset(HANDLE, FLOAT, FLOAT)` | 928 | Функция устанавливает смещение плоскости проецирования относительно положения камеры в пространстве. Используется при стерео визуализации. |
| [Camera_SetNearClipDistance](../help/topics/Camera_SetNearClipDistance.md) | `Camera_SetNearClipDistance(HANDLE, FLOAT)` | 920 | Функция устанавливает ближнюю плоскость отсечения камеры. |
| [Camera_SetPolygonMode](../help/topics/Camera_SetPolygonMode.md) | `Camera_SetPolygonMode(HANDLE, FLOAT)` | 926 | Функция устанавливает режим отрисовки полигонов. |
| [Camera_SetProjectionType](../help/topics/Camera_SetProjectionType.md) | `Camera_SetProjectionType(HANDLE, FLOAT)` | 924 | Функция устанавливает тип проецирования. |
| [Collision_GetDistance](../help/topics/Collision_GetDistance.md) | `FLOAT Collision_GetDistance(FLOAT)` | 933 | Функция для получения расстояния до определенного пересечения. |
| [Collision_GetObject](../help/topics/Collision_GetObject.md) | `HANDLE Collision_GetObject(FLOAT)` | 934 | Функция для получения определенного объекта пересечения из списка. |
| [Collision_GetResultCount](../help/topics/Collision_GetResultCount.md) | `FLOAT Collision_GetResultCount()` | 932 | Функция для получения числа пересеченных лучом объектов. |
| [Collision_RayCast](../help/topics/Collision_RayCast.md) | `Collision_RayCast(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 930 | Функция запускает алгоритм вычисления пересечений луча и объектов сцены. |
| [Collision_Sort](../help/topics/Collision_Sort.md) | `Collision_Sort()` | 931 | Функция для сортировки списка результатов расчета столкновения. Сортировка производится по расстоянию до пересечения. |
| [Compositor_GetEnabled](../help/topics/Compositor_GetEnabled.md) | `FLOAT Compositor_GetEnabled(HANDLE)` | 1205 | Функция для определения режима работы композитора: включен или выключен. |
| [Compositor_SetEnabled](../help/topics/Compositor_SetEnabled.md) | `Compositor_SetEnabled(HANDLE, FLOAT)` | 1204 | Функция для включения и выключения композитора. |
| [Entity_Create](../help/topics/Entity_Create.md) | `HANDLE Entity_Create(HANDLE, STRING, STRING)` | 935 | Функция используется для создания трехмерной модели. |
| [Entity_Destroy](../help/topics/Entity_Destroy.md) | `Entity_Destroy(HANDLE)` | 936 | Функция используется для удаления трехмерной модели. |
| [Entity_GetAnimationState](../help/topics/Entity_GetAnimationState.md) | `HANDLE Entity_GetAnimationState(HANDLE, STRING)` | 938 | Функция для получения состояния анимации трехмерной модели. |
| [Entity_GetBone](../help/topics/Entity_GetBone.md) | `HANDLE Entity_GetBone(HANDLE, STRING)` | 939 | Функция для получения кости скелета трехмерной модели. |
| [Entity_GetIndexCount](../help/topics/Entity_GetIndexCount.md) | `FLOAT Entity_GetIndexCount(HANDLE, FLOAT)` | 1294 | Функция для получения количества индексов в секции трехмерной модели. Через количество индексов можно вычислить такие характеристики как количество ребер (равно количеству индексов - 1) и граней (количество индексов разделить на 3) модели. |
| [Entity_GetSubEntityCount](../help/topics/Entity_GetSubEntityCount.md) | `FLOAT Entity_GetSubEntityCount(HANDLE)` | 1268 | Функция для получения количества секций в трехмерной модели. |
| [Entity_GetVertexCount](../help/topics/Entity_GetVertexCount.md) | `FLOAT Entity_GetVertexCount(HANDLE, FLOAT)` | 1293 | Функция для получения количества вершин в секции трехмерной модели. |
| [Entity_SetMaterial](../help/topics/Entity_GetMaterial.md) | `Entity_SetMaterial(HANDLE, FLOAT, STRING)` | 937 | Получение имени материала секции трехмерной модели по индексу секции. |
| [Light_Create](../help/topics/Light_Create.md) | `HANDLE Light_Create(HANDLE, STRING)` | 940 | Функция используется для создания источника света. |
| [Light_Destroy](../help/topics/Light_Destroy.md) | `Light_Destroy(HANDLE)` | 941 | Функция используется для удаления источника света. |
| [Light_GetPowerScale](../help/topics/Light_GetPowerScale.md) | `FLOAT Light_GetPowerScale(HANDLE)` | 948 | Функция для получения интенсивности источника света. |
| [Light_GetType](../help/topics/Light_GetType.md) | `FLOAT Light_GetType (FLOAT Light)` |  | Функция для получения типа источника света. |
| [Light_SetAttenuation](../help/topics/Light_SetAttenuation.md) | `Light_SetAttenuation(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 942 | Функция устанавливает параметры затухания источника света. |
| [Light_SetDiffuseColour](../help/topics/Light_SetDiffuseColour.md) | `Light_SetDiffuseColour(HANDLE, FLOAT, FLOAT, FLOAT)` | 944 | Функция устанавливает цвет рассеянного компонента освещения источника света. |
| [Light_SetPowerScale](../help/topics/Light_SetPowerScale.md) | `Light_SetPowerScale(HANDLE, FLOAT)` | 947 | Функция устанавливает интенсивность источника света. |
| [Light_SetSpecularColour](../help/topics/Light_SetSpecularColour.md) | `Light_SetSpecularColour(HANDLE, FLOAT, FLOAT, FLOAT)` | 945 | Функция устанавливает цвет отраженного компонента освещения источника света. |
| [Light_SetSpotlightRange](../help/topics/Light_SetSpotlightRange.md) | `Light_SetSpotlightRange(HANDLE, FLOAT, FLOAT, FLOAT)` | 943 | Функция устанавливает параметры направленного источника света. |
| [Light_SetType](../help/topics/Light_SetType.md) | `Light_SetType(HANDLE, FLOAT)` | 946 | Функция устанавливает тип источника света. |
| [ManualObject_Begin](../help/topics/ManualObject_Begin.md) | `ManualObject_Begin(HANDLE, STRING, FLOAT)` | 956 | Вызов данной функции инициирует процесс создания секции трехмерной модели. После чего можно вводить геометрические данные с помощью функций данного класса. |
| [ManualObject_Clear](../help/topics/ManualObject_Clear.md) | `ManualObject_Clear(HANDLE)` | 951 | Функция используется для удаления всех геометрических данных пользовательской модели, сама модель не удаляется. |
| [ManualObject_Colour](../help/topics/ManualObject_Colour.md) | `ManualObject_Colour(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 960 | Функция устанавливает цвет добавленной вершины в пользовательской трехмерной модели. |
| [ManualObject_ConvertToMesh](../help/topics/ManualObject_ConvertToMesh.md) | `ManualObject_ConvertToMesh(HANDLE, STRING)` | 963 | Функция сохраняет содержимое пользовательской модели в объекте-ресурсе трехмерной модели. В дальнейшем данный ресурс может быть использован объектами класса Entity. |
| [ManualObject_Create](../help/topics/ManualObject_Create.md) | `HANDLE ManualObject_Create(HANDLE, STRING)` | 949 | Функция используется для создания пользовательской трехмерной модели. |
| [ManualObject_Destroy](../help/topics/ManualObject_Destroy.md) | `ManualObject_Destroy(HANDLE)` | 950 | Функция используется для удаления пользовательской трехмерной модели. |
| [ManualObject_End](../help/topics/ManualObject_End.md) | `ManualObject_End(HANDLE)` | 957 | Вызов данной функции завершает процесс описания секции трехмерной модели. |
| [ManualObject_EstimateIndexCount](../help/topics/ManualObject_EstimateIndexCount.md) | `ManualObject_EstimateIndexCount(HANDLE, FLOAT)` | 953 | Данная функция позволяет заранее определить количество индексов в будущей трехмерной модели, тем самым позволяя сократить нежелательные перераспределения памяти. Использование функции не обязательно, но позволяет немного ускорить работу системы. |
| [ManualObject_EstimateVertexCount](../help/topics/ManualObject_EstimateVertexCount.md) | `ManualObject_EstimateVertexCount(HANDLE, FLOAT)` | 952 | Данная функция позволяет заранее определить количество вершин в будущей трехмерной модели, тем самым позволяя сократить нежелательные перераспределения памяти. Использование функции не обязательно, но позволяет немного ускорить работу системы. |
| [ManualObject_GetDynamic](../help/topics/ManualObject_GetDynamic.md) | `FLOAT ManualObject_GetDynamic(HANDLE)` | 955 | Функция для получения флага динамического режима обновления трехмерной модели. |
| [ManualObject_Index](../help/topics/ManualObject_Index.md) | `ManualObject_Index(HANDLE, FLOAT)` | 962 | Функция добавляет индекс вершины в пользовательскую трехмерную модель. |
| [ManualObject_Normal](../help/topics/ManualObject_Normal.md) | `ManualObject_Normal(HANDLE, FLOAT, FLOAT, FLOAT)` | 959 | Функция устанавливает нормаль добавленной вершины в пользовательской трехмерной модели. |
| [ManualObject_Position](../help/topics/ManualObject_Position.md) | `ManualObject_Position(HANDLE, FLOAT, FLOAT, FLOAT)` | 958 | Функция добавляет вершину в пользовательскую трехмерную модель. |
| [ManualObject_SetDynamic](../help/topics/ManualObject_SetDynamic.md) | `ManualObject_SetDynamic(HANDLE, FLOAT)` | 954 | Функция включает или выключает динамический режим обновления трехмерной модели. |
| [ManualObject_TextureCoord](../help/topics/ManualObject_TextureCoord.md) | `ManualObject_TextureCoord(HANDLE, FLOAT, FLOAT)` | 961 | Функция устанавливает текстурные координаты добавленной вершины в пользовательской трехмерной модели. |
| [Material_Create](../help/topics/Material_Create.md) | `HANDLE Material_Create(STRING)` | 964 | Функция для создания материала. |
| [Material_Get](../help/topics/Material_Get.md) | `HANDLE Material_Get(STRING)` | 965 | Функция для получения объекта материала по его имени. |
| [Material_GetBestTechnique](../help/topics/Material_GetBestTechnique.md) | `HANDLE Material_GetBestTechnique(HANDLE)` | 966 | Функция для получения используемой техники рендеринга в материале. |
| [Material_GetName](../help/topics/Material_GetName.md) | `Material_GetName(HANDLE, &STRING)` | 1219 | Получить имя материала по его дескриптору |
| [Material_GetTechniqueByIndex](../help/topics/Material_GetTechniqueByIndex.md) | `HANDLE Material_GetTechniqueByIndex(HANDLE, FLOAT)` | 968 | Функция для получения техники рендеринга в материале по её индексу. |
| [Material_GetTechniqueByName](../help/topics/Material_GetTechniqueByName.md) | `HANDLE Material_GetTechniqueByName(HANDLE, STRING)` | 967 | Функция для получения техники рендеринга в материале по её имени. |
| [Movable_GetBoundingBox](../help/topics/Movable_GetBoundingBox.md) | `Movable_GetBoundingBox(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1213 | Функция для получения обрамляющего переллелепипеда для перемещаемого объекта (в системе координат родительского узла). |
| [Movable_GetCastShadows](../help/topics/Movable_GetCastShadows.md) | `FLOAT Movable_GetCastShadows(HANDLE)` | 972 | Функция для получения флага отбрасывания теней перемещаемым объектом. |
| [Movable_GetName](../help/topics/Movable_GetName.md) | `Movable_GetName(HANDLE, &STRING)` | 1212 | Функция для получения имени перемещамого объекта. |
| [Movable_GetParent](../help/topics/Movable_GetParent.md) | `HANDLE Movable_GetParent(HANDLE)` | 970 | Функция для получения родительского узла у перемещаемого объекта сцены. |
| [Movable_GetVisible](../help/topics/Movable_GetVisible.md) | `FLOAT Movable_GetVisible(HANDLE)` | 974 | Функция для получения флага видимости перемещаемого объекта. |
| [Movable_SetCastShadows](../help/topics/Movable_SetCastShadows.md) | `Movable_SetCastShadows(HANDLE, FLOAT)` | 971 | Функция для установки флага отбрасывания теней перемещаемым объектом. |
| [Movable_SetParent](../help/topics/Movable_SetParent.md) | `Movable_SetParent(HANDLE, HANDLE)` | 969 | Функция для установки родительского узла для перемещаемого объекта сцены. |
| [Movable_SetVisible](../help/topics/Movable_SetVisible.md) | `Movable_SetVisible(HANDLE, FLOAT)` | 973 | Функция для установки флага видимости перемещаемого объекта. |
| [MovableText_Create](../help/topics/MovableText_Create.md) | `HANDLE MovableText_Create(STRING, STRING, STRING)` | 1220 | Конструктор объекта класса MovableText. |
| [MovableText_Destroy](../help/topics/MovableText_Destroy.md) | `MovableText_Destroy(HANDLE)` | 1221 | Уничтожение объекта класса MovableText. |
| [MovableText_GetCaption](../help/topics/MovableText_GetCaption.md) | `MovableText_GetCaption(HANDLE, &STRING)` | 1225 | Получение отображаемого текста. |
| [MovableText_GetCharacterHeight](../help/topics/MovableText_GetCharacterHeight.md) | `FLOAT MovableText_GetCharacterHeight(HANDLE)` | 1229 | Получение размера (высоты строки) отображаемого текста. |
| [MovableText_GetColor](../help/topics/MovableText_GetColor.md) | `MovableText_GetColor(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1227 | Получение цвета отображаемого текста. |
| [MovableText_GetFontName](../help/topics/MovableText_GetFontName.md) | `MovableText_GetFontName(HANDLE, &STRING)` | 1223 | Получение имени шрифта. |
| [MovableText_GetGlobalTranslation](../help/topics/MovableText_GetGlobalTranslation.md) | `MovableText_GetGlobalTranslation(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1235 | Получение значения глобального смещения текста. |
| [MovableText_GetLocalTranslation](../help/topics/MovableText_GetLocalTranslation.md) | `MovableText_GetLocalTranslation(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1237 | Получение значения локального смещения текста относительно центра системы координат узла сцены. |
| [MovableText_GetSpaceWidth](../help/topics/MovableText_GetSpaceWidth.md) | `FLOAT MovableText_GetSpaceWidth(HANDLE)` | 1231 | Получение ширины символа пробела. |
| [MovableText_GetTextAlignment](../help/topics/MovableText_GetTextAlignment.md) | `MovableText_GetTextAlignment(HANDLE, &FLOAT, &FLOAT)` | 1233 | Получение значений выравнивания текста. |
| [MovableText_SetCaption](../help/topics/MovableText_SetCaption.md) | `MovableText_SetCaption(HANDLE, STRING)` | 1224 | Установка отображаемого текста после создания объекта. |
| [MovableText_SetCharacterHeight](../help/topics/MovableText_SetCharacterHeight.md) | `MovableText_SetCharacterHeight(HANDLE, FLOAT)` | 1228 | Установка размера (высоты строки) отображаемого текста. |
| [MovableText_SetColor](../help/topics/MovableText_SetColor.md) | `MovableText_SetColor(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1226 | Установка цвета отображаемого текста. |
| [MovableText_SetFontName](../help/topics/MovableText_SetFontName.md) | `MovableText_SetFontName(HANDLE, STRING)` | 1222 | Установка шрифта после создания объекта. |
| [MovableText_SetGlobalTranslation](../help/topics/MovableText_SetGlobalTranslation.md) | `MovableText_SetGlobalTranslation(HANDLE, FLOAT, FLOAT, FLOAT)` | 1234 | Установка глобального смещения текста. |
| [MovableText_SetLocalTranslation](../help/topics/MovableText_SetLocalTranslation.md) | `MovableText_SetLocalTranslation(HANDLE, FLOAT, FLOAT, FLOAT)` | 1236 | Установка локального смещения текста относительно центра системы координат узла сцены. |
| [MovableText_SetSpaceWidth](../help/topics/MovableText_SetSpaceWidth.md) | `MovableText_SetSpaceWidth(HANDLE, FLOAT)` | 1230 | Установка ширины символа пробела. |
| [MovableText_SetTextAlignment](../help/topics/MovableText_SetTextAlignment.md) | `MovableText_SetTextAlignment(HANDLE, FLOAT, FLOAT)` | 1232 | Установка выравнивания текста относительно центра системы координат узла сцены. |
| [Node_AddChild](../help/topics/Node_AddChild.md) | `Node_AddChild(HANDLE, HANDLE)` | 987 | Функция для добавления дочернего узла. В большинстве случаев пользователь не должен пользоваться данной функцией, следует использовать функцию [Node_SetParent](Node_SetParent.md). |
| [Node_GetChild](../help/topics/Node_GetChild.md) | `HANDLE Node_GetChild(HANDLE, FLOAT)` | 1211 | Функция для получения дочернего узла по его индексу. |
| [Node_GetDerivedPosition](../help/topics/Node_GetDerivedPosition.md) | `Node_GetDerivedPosition(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1284 | Функция для получения глобальных координат узла в мировой системе координат. |
| [Node_GetDerivedRotationQuaternion](../help/topics/Node_GetDerivedRotationQuaternion.md) | `Node_GetDerivedRotationQuaternion(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1286 | Функция для получения глобальной ориентации (поворота) узла в мировой системе координат. |
| [Node_GetDerivedScale](../help/topics/Node_GetDerivedScale.md) | `Node_GetDerivedScale(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1285 | Функция для получения глобального масштаба узла в мировой системе координат. |
| [Node_GetName](../help/topics/Node_GetName.md) | `Node_GetName(HANDLE, &STRING)` | 1206 | Функция для получения имени узла. |
| [Node_GetNumChildren](../help/topics/Node_GetNumChildren.md) | `FLOAT Node_GetNumChildren(HANDLE)` | 1210 | Функция для получения количества дочерних узлов. |
| [Node_GetParent](../help/topics/Node_GetParent.md) | `HANDLE Node_GetParent(HANDLE)` | 986 | Функция для получения родительского узла. |
| [Node_GetPosition](../help/topics/Node_GetPosition.md) | `Node_GetPosition(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1207 | Функция для получения координат узла (в системе координат родительского узла). |
| [Node_GetRotationQuaternion](../help/topics/Node_GetRotationQuaternion.md) | `Node_GetRotationQuaternion(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1209 | Функция для получения ориентации (поворота) узла (в системе координат родительского узла). |
| [Node_GetScale](../help/topics/Node_GetScale.md) | `Node_GetScale(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1208 | Функция для получения масштаба узла (в системе координат родительского узла). |
| [Node_RemoveChild](../help/topics/Node_RemoveChild.md) | `Node_RemoveChild(HANDLE, HANDLE)` | 988 | Функция для удаления дочернего узла. В большинстве случаев пользователь не должен пользоваться данной функцией, следует использовать функцию [Node_SetParent](Node_SetParent.md). |
| [Node_SetParent](../help/topics/Node_SetParent.md) | `Node_SetParent(HANDLE, HANDLE)` | 985 | Функция для установки родительского узла. |
| [Node_SetPosition](../help/topics/Node_SetPosition.md) | `Node_SetPosition(HANDLE, FLOAT, FLOAT, FLOAT)` | 975 | Функция для установки позиции узла в системе координат родительского узла. |
| [Node_SetRotationAxis](../help/topics/Node_SetRotationAxis.md) | `Node_SetRotationAxis(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 982 | Функция для установки ориентации узла с помощью оси и поворота вокруг нее. |
| [Node_SetRotationEulerXYZ](../help/topics/Node_SetRotationEulerXYZ.md) | `Node_SetRotationEulerXYZ(HANDLE, FLOAT, FLOAT, FLOAT)` | 976 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: X, Y, Z. |
| [Node_SetRotationEulerXZY](../help/topics/Node_SetRotationEulerXZY.md) | `Node_SetRotationEulerXZY(HANDLE, FLOAT, FLOAT, FLOAT)` | 977 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: X, Z, Y. |
| [Node_SetRotationEulerYXZ](../help/topics/Node_SetRotationEulerYXZ.md) | `Node_SetRotationEulerYXZ(HANDLE, FLOAT, FLOAT, FLOAT)` | 978 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: Y, X, Z. |
| [Node_SetRotationEulerYZX](../help/topics/Node_SetRotationEulerYZX.md) | `Node_SetRotationEulerYZX(HANDLE, FLOAT, FLOAT, FLOAT)` | 979 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: Y, Z, X. |
| [Node_SetRotationEulerZXY](../help/topics/Node_SetRotationEulerZXY.md) | `Node_SetRotationEulerZXY(HANDLE, FLOAT, FLOAT, FLOAT)` | 980 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: Z, X, Y. |
| [Node_SetRotationEulerZYX](../help/topics/Node_SetRotationEulerZYX.md) | `Node_SetRotationEulerZYX(HANDLE, FLOAT, FLOAT, FLOAT)` | 981 | Функция для установки ориентации узла с помощью углов Эйлера, при этом поворот вокруг осей осуществляется в следующем порядке: Z, Y, X. |
| [Node_SetRotationQuaternion](../help/topics/Node_SetRotationQuaternion.md) | `Node_SetRotationQuaternion(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 983 | Функция для установки ориентации узла с помощью кватерниона. |
| [Node_SetScale](../help/topics/Node_SetScale.md) | `Node_SetScale(HANDLE, FLOAT, FLOAT, FLOAT)` | 984 | Функция для установки масштаба узла. |
| [Overlay_AddChild](../help/topics/Overlay_AddChild.md) | `Overlay_AddChild(HANDLE, HANDLE)` | 1249 | Добавление дочернего элемента-контейнера в оверлей. |
| [Overlay_Create](../help/topics/Overlay_Create.md) | `HANDLE Overlay_Create(STRING)` | 1238 | Конструктор объекта класса Overlay. |
| [Overlay_Get](../help/topics/Overlay_Get.md) | `HANDLE Overlay_Get(STRING)` | 989 | Функция для получения объекта оверлея по его имени. |
| [Overlay_GetName](../help/topics/Overlay_GetName.md) | `Overlay_GetName(HANDLE, &STRING)` | 1239 | Получение имени оверлея. |
| [Overlay_GetRotate](../help/topics/Overlay_GetRotate.md) | `FLOAT Overlay_GetRotate(HANDLE)` | 1248 | Получение угла поворота для оверлея. |
| [Overlay_GetScale](../help/topics/Overlay_GetScale.md) | `Overlay_GetScale(HANDLE, &FLOAT, &FLOAT)` | 1244 | Получение масштаба для оверлея. |
| [Overlay_GetScroll](../help/topics/Overlay_GetScroll.md) | `Overlay_GetScroll(HANDLE, &FLOAT, &FLOAT)` | 1246 | Получение относительного смещения для оверлея. |
| [Overlay_GetVisible](../help/topics/Overlay_GetVisible.md) | `FLOAT Overlay_GetVisible(HANDLE)` | 991 | Функция для получения флага видимости оверлея. |
| [Overlay_GetZOrder](../help/topics/Overlay_GetZOrder.md) | `FLOAT Overlay_GetZOrder(HANDLE)` | 1242 | Получение Z-порядка для оверлея. |
| [Overlay_RemoveChild](../help/topics/Overlay_RemoveChild.md) | `Overlay_RemoveChild(HANDLE, HANDLE)` | 1250 | Удаление дочернего элемента-контейнера из оверлея. |
| [Overlay_SetRotate](../help/topics/Overlay_SetRotate.md) | `Overlay_SetRotate(HANDLE, FLOAT)` | 1247 | Установка угла поворота для оверлея. |
| [Overlay_SetScale](../help/topics/Overlay_SetScale.md) | `Overlay_SetScale(HANDLE, FLOAT, FLOAT)` | 1243 | Установка масштаба для оверлея. |
| [Overlay_SetScroll](../help/topics/Overlay_SetScroll.md) | `Overlay_SetScroll(HANDLE, FLOAT, FLOAT)` | 1245 | Установка относительного смещения для оверлея. |
| [Overlay_SetVisible](../help/topics/Overlay_SetVisible.md) | `Overlay_SetVisible(HANDLE, FLOAT)` | 990 | Функция для установки флага видимости оверлея. |
| [Overlay_SetZOrder](../help/topics/Overlay_SetZOrder.md) | `Overlay_SetZOrder(HANDLE, FLOAT)` | 1241 | Установка Z-порядка для оверлея. |
| [OverlayContainer_AddChild](../help/topics/OverlayContainer_AddChild.md) | `OverlayContainer_AddChild(HANDLE, HANDLE)` | 1251 | Добавление дочернего элемента в элемент-контейнер. |
| [OverlayContainer_GetChild](../help/topics/OverlayContainer_GetChild.md) | `HANDLE OverlayContainer_GetChild(HANDLE, FLOAT)` | 1253 | Получение дочернего элемента у элемента-контейнера по индексу. |
| [OverlayContainer_GetChildCount](../help/topics/OverlayContainer_GetChildCount.md) | `FLOAT OverlayContainer_GetChildCount(HANDLE)` | 1254 | Получение количества элементов в элементе-контейнере. |
| [OverlayContainer_RemoveChild](../help/topics/OverlayContainer_RemoveChild.md) | `OverlayContainer_RemoveChild(HANDLE, HANDLE)` | 1252 | Удаление дочернего элемента из элемента-контейнера. |
| [OverlayElement_Create](../help/topics/OverlayElement_Create.md) | `HANDLE OverlayElement_Create(STRING, STRING)` | 1255 | Создание элемента оверлея. |
| [OverlayElement_Get](../help/topics/OverlayElement_Get.md) | `HANDLE OverlayElement_Get(STRING)` | 992 | Функция для получения элемента оверлея по его имени. |
| [OverlayElement_GetAlignment](../help/topics/OverlayElement_GetAlignment.md) | `OverlayElement_GetAlignment(HANDLE, &FLOAT, &FLOAT)` | 1264 | Получение режима выравнивания элемента. |
| [OverlayElement_GetCaption](../help/topics/OverlayElement_GetCaption.md) | `OverlayElement_GetCaption(HANDLE, &STRING)` | 1259 | Получение текста элемента. |
| [OverlayElement_GetColour](../help/topics/OverlayElement_GetColour.md) | `OverlayElement_GetColour(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1258 | Получение цвета и прозрачности элемента. |
| [OverlayElement_GetMaterialName](../help/topics/OverlayElement_GetMaterialName.md) | `OverlayElement_GetMaterialName(HANDLE, &STRING)` | 1260 | Получение имени материала элемента. |
| [OverlayElement_GetMetricsMode](../help/topics/OverlayElement_GetMetricsMode.md) | `FLOAT OverlayElement_GetMetricsMode(HANDLE)` | 1262 | Получение режима системы координат (относительная / абсолютная). |
| [OverlayElement_GetParent](../help/topics/OverlayElement_GetParent.md) | `HANDLE OverlayElement_GetParent(HANDLE)` | 1266 | Получение родительского контейнера для элемента. |
| [OverlayElement_GetPosition](../help/topics/OverlayElement_GetPosition.md) | `OverlayElement_GetPosition(HANDLE, &FLOAT, &FLOAT)` | 1256 | Получение координат элемента. В зависимости от заданного режима координаты могут быть как абсолютные, так и относительные (см. [OverlayElement_SetMetricsMode](OverlayElement_SetMetricsMode.md)). |
| [OverlayElement_GetSize](../help/topics/OverlayElement_GetSize.md) | `OverlayElement_GetSize(HANDLE, &FLOAT, &FLOAT)` | 1257 | Получение размеров элемента. В зависимости от заданного режима размеры могут быть как абсолютные, так и относительные (см. [OverlayElement_SetMetricsMode](OverlayElement_SetMetricsMode.md)). |
| [OverlayElement_GetVisible](../help/topics/OverlayElement_GetVisible.md) | `FLOAT OverlayElement_GetVisible(HANDLE)` | 994 | Функция для получения флага видимости элемента  оверлея. |
| [OverlayElement_IsContainer](../help/topics/OverlayElement_IsContainer.md) | `FLOAT OverlayElement_IsContainer(HANDLE)` | 1265 | Проверка является ли элемент контейнером(класс OverlayContainer). |
| [OverlayElement_SetAlignment](../help/topics/OverlayElement_SetAlignment.md) | `OverlayElement_SetAlignment(HANDLE, FLOAT, FLOAT)` | 1263 | Установка режима выравнивания элемента. |
| [OverlayElement_SetCaption](../help/topics/OverlayElement_SetCaption.md) | `OverlayElement_SetCaption(HANDLE, STRING)` | 998 | Функция для установки текста на элементе оверлея (если элемент имеет возможность вывода текста). |
| [OverlayElement_SetColour](../help/topics/OverlayElement_SetColour.md) | `OverlayElement_SetColour(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 997 | Функция для установки цвета элемента оверлея. |
| [OverlayElement_SetMaterialName](../help/topics/OverlayElement_SetMaterialName.md) | `OverlayElement_SetMaterialName(HANDLE, STRING)` | 999 | Функция для установки материала элемента оверлея. |
| [OverlayElement_SetMetricsMode](../help/topics/OverlayElement_SetMetricsMode.md) | `OverlayElement_SetMetricsMode(HANDLE, FLOAT)` | 1261 | Установка режима системы координат (относительная / абсолютная). |
| [OverlayElement_SetPosition](../help/topics/OverlayElement_SetPosition.md) | `OverlayElement_SetPosition(HANDLE, FLOAT, FLOAT)` | 995 | Функция для установки позиции элемента оверлея. |
| [OverlayElement_SetSize](../help/topics/OverlayElement_SetSize.md) | `OverlayElement_SetSize(HANDLE, FLOAT, FLOAT)` | 996 | Функция для установки размера элемента оверлея. |
| [OverlayElement_SetVisible](../help/topics/OverlayElement_SetVisible.md) | `OverlayElement_SetVisible(HANDLE, FLOAT)` | 993 | Функция для установки флага видимости элемента оверлея. |
| [ParticleSystem_Create](../help/topics/ParticleSystem_Create.md) | `HANDLE ParticleSystem_Create(HANDLE, STRING, STRING)` | 1000 | Функция для создания системы частиц. |
| [ParticleSystem_Destroy](../help/topics/ParticleSystem_Destroy.md) | `ParticleSystem_Destroy(HANDLE)` | 1001 | Функция для удаления системы частиц. |
| [ParticleSystem_GetName](../help/topics/ParticleSystem_GetName.md) | `ParticleSystem_GetName(HANDLE, STRING)` | 1309 | Получение названия системы частиц. |
| [ParticleSystem_GetParent](../help/topics/ParticleSystem_GetParent.md) | `HANDLE ParticleSystem_GetParent(HANDLE)` | 1308 | Функция для получения родительского узла у системы частиц. |
| [ParticleSystem_GetVisible](../help/topics/ParticleSystem_GetVisible.md) | `FLOAT ParticleSystem_GetVisible(HANDLE)` | 1306 | Функция для получения флага видимости системы частиц. |
| [ParticleSystem_SetParent](../help/topics/ParticleSystem_SetParent.md) | `ParticleSystem_SetParent(HANDLE, HANDLE)` | 1307 | Функция для установки родительского узла для системы частиц. |
| [ParticleSystem_SetVisible](../help/topics/ParticleSystem_SetVisible.md) | `ParticleSystem_SetVisible(HANDLE, FLOAT)` | 1305 | Функция для установки флага видимости системы частиц. |
| [Pass_Create](../help/topics/Pass_Create.md) | `HANDLE Pass_Create(HANDLE, STRING)` | 1002 | Функция для создания итерации для техники материала. |
| [Pass_GetAlphaRejectSettings](../help/topics/Pass_GetAlphaRejectSettings.md) | `Pass_GetAlphaRejectSettings(HANDLE, &FLOAT, &FLOAT)` | 1283 | Функция для получения настроек альфа-тестирования. |
| [Pass_GetAmbient](../help/topics/Pass_GetAmbient.md) | `Pass_GetAmbient(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1269 | Функция для получения цвета отраженного материалом фонового освещения. |
| [Pass_GetColourWriteEnabled](../help/topics/Pass_GetColourWriteEnabled.md) | `FLOAT Pass_GetColourWriteEnabled(HANDLE)` | 1278 | Получение флага записи в буфер кадра. |
| [Pass_GetCullingMode](../help/topics/Pass_GetCullingMode.md) | `FLOAT Pass_GetCullingMode(HANDLE)` | 1279 | Получение режима отсечения полигонов. |
| [Pass_GetDepthCheckEnabled](../help/topics/Pass_GetDepthCheckEnabled.md) | `FLOAT Pass_GetDepthCheckEnabled(HANDLE)` | 1275 | Получение флага проверки буфера глубины. |
| [Pass_GetDepthFunction](../help/topics/Pass_GetDepthFunction.md) | `FLOAT Pass_GetDepthFunction(HANDLE)` | 1277 | Получение типа функции проверки буфера глубины. |
| [Pass_GetDepthWriteEnabled](../help/topics/Pass_GetDepthWriteEnabled.md) | `FLOAT Pass_GetDepthWriteEnabled(HANDLE)` | 1276 | Получение флага записи в буфер глубины. |
| [Pass_GetDiffuse](../help/topics/Pass_GetDiffuse.md) | `Pass_GetDiffuse(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1270 | Функция для получения цвета рассеянного материалом освещения. |
| [Pass_GetLightingEnabled](../help/topics/Pass_GetLightingEnabled.md) | `FLOAT Pass_GetLightingEnabled(HANDLE)` | 1280 | Получение флага вычисления освещения для материала.. |
| [Pass_GetPolygonMode](../help/topics/Pass_GetPolygonMode.md) | `FLOAT Pass_GetPolygonMode(HANDLE)` | 1282 | Получение режима отрисовки полигонов. |
| [Pass_GetSceneBlending](../help/topics/Pass_GetSceneBlending.md) | `Pass_GetSceneBlending(HANDLE, &FLOAT, &FLOAT)` | 1274 | Функция для получения режима смешивания для материала. |
| [Pass_GetSelfIllumination](../help/topics/Pass_GetSelfIllumination.md) | `Pass_GetSelfIllumination(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1273 | Функция для получения цвета собственного фонового освещения для материала. |
| [Pass_GetShadingMode](../help/topics/Pass_GetShadingMode.md) | `FLOAT Pass_GetShadingMode(HANDLE)` | 1281 | Получение режима заполнения полигонов. |
| [Pass_GetShininess](../help/topics/Pass_GetShininess.md) | `FLOAT Pass_GetShininess(HANDLE)` | 1272 | Функция для получения коэффициента отражения освещения. |
| [Pass_GetSpecular](../help/topics/Pass_GetSpecular.md) | `Pass_GetSpecular(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1271 | Функция для получения цвета отраженного материалом освещения. |
| [Pass_GetTextureUnitStateByIndex](../help/topics/Pass_GetTextureUnitStateByIndex.md) | `HANDLE Pass_GetTextureUnitStateByIndex(HANDLE, FLOAT)` | 1019 | Функция для получения текстурного блока по индексу. |
| [Pass_GetTextureUnitStateByName](../help/topics/Pass_GetTextureUnitStateByName.md) | `HANDLE Pass_GetTextureUnitStateByName(HANDLE, STRING)` | 1018 | Функция для получения текстурного блока по имени. |
| [Pass_SetAlphaRejectSettings](../help/topics/Pass_SetAlphaRejectSettings.md) | `Pass_SetAlphaRejectSettings(HANDLE, FLOAT, FLOAT)` | 1017 | Функция устанавливает настройки альфа-тестирования. |
| [Pass_SetAmbient](../help/topics/Pass_SetAmbient.md) | `Pass_SetAmbient(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1003 | Функция устанавливает цвет отраженного материалом фонового освещения. |
| [Pass_SetColourWriteEnabled](../help/topics/Pass_SetColourWriteEnabled.md) | `Pass_SetColourWriteEnabled(HANDLE, FLOAT)` | 1012 | Функция устанавливает флаг записи в буфер кадра. |
| [Pass_SetCullingMode](../help/topics/Pass_SetCullingMode.md) | `Pass_SetCullingMode(HANDLE, FLOAT)` | 1013 | Функция устанавливает режим отсечения полигонов. |
| [Pass_SetDepthCheckEnabled](../help/topics/Pass_SetDepthCheckEnabled.md) | `Pass_SetDepthCheckEnabled(HANDLE, FLOAT)` | 1009 | Функция устанавливает флаг проверки буфера глубины. |
| [Pass_SetDepthFunction](../help/topics/Pass_SetDepthFunction.md) | `Pass_SetDepthFunction(HANDLE, FLOAT)` | 1011 | Функция устанавливает тип функции проверки буфера глубины. |
| [Pass_SetDepthWriteEnabled](../help/topics/Pass_SetDepthWriteEnabled.md) | `Pass_SetDepthWriteEnabled(HANDLE, FLOAT)` | 1010 | Функция устанавливает флаг записи в буфер глубины. |
| [Pass_SetDiffuse](../help/topics/Pass_SetDiffuse.md) | `Pass_SetDiffuse(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1004 | Функция устанавливает цвет рассеянного материалом освещения. |
| [Pass_SetLightingEnabled](../help/topics/Pass_SetLightingEnabled.md) | `Pass_SetLightingEnabled(HANDLE, FLOAT)` | 1014 | Функция устанавливает флаг вычисления освещения для материала. |
| [Pass_SetPolygonMode](../help/topics/Pass_SetPolygonMode.md) | `Pass_SetPolygonMode(HANDLE, FLOAT)` | 1016 | Функция устанавливает режим отрисовки полигонов. |
| [Pass_SetSceneBlending](../help/topics/Pass_SetSceneBlending.md) | `Pass_SetSceneBlending(HANDLE, FLOAT, FLOAT)` | 1008 | Функция устанавливает режим смешивания для материала. |
| [Pass_SetSelfIllumination](../help/topics/Pass_SetSelfIllumination.md) | `Pass_SetSelfIllumination(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1007 | Функция устанавливает цвет собственного фонового освещения для материала. |
| [Pass_SetShadingMode](../help/topics/Pass_SetShadingMode.md) | `Pass_SetShadingMode(HANDLE, FLOAT)` | 1015 | Функция устанавливает режим заполнения полигонов. |
| [Pass_SetShininess](../help/topics/Pass_SetShininess.md) | `Pass_SetShininess(HANDLE, FLOAT)` | 1006 | Функция устанавливает коэффициент отражения освещения. Влияет на величину бликов. |
| [Pass_SetSpecular](../help/topics/Pass_SetSpecular.md) | `Pass_SetSpecular(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1005 | Функция устанавливает цвет отраженного материалом освещения. |
| [RenderTexture_Create](../help/topics/RenderWindow_Create.md) | `HANDLE RenderTexture_Create(STRING, FLOAT, FLOAT)` | 1020 | Функция создания окна рендеринга. |
| [RenderTexture_CreateEx](../help/topics/RenderWindow_CreateEx.md) | `HANDLE RenderTexture_CreateEx (HANDLE a_Window, FLOAT a_Fullscreen, FLOAT a_FSAA, FLOAT a_VSync)` |  | Функция создания окна рендеринга с расширенными параметрами. |
| [RenderTexture_Destroy](../help/topics/RenderTexture_Destroy.md) | `RenderTexture_Destroy(HANDLE)` | 1021 | Функция удаления текстуры для рендеринга. |
| [RenderWindow_Get](../help/topics/RenderWindow_Get.md) | `HANDLE RenderWindow_Get(FLOAT)` | 1302 | Функция для получения окна рендеринга по индексу. |
| [RenderWindow_GetCount](../help/topics/RenderWindow_GetCount.md) | `FLOAT RenderWindow_GetCount()` | 1301 | Функция для получения количества окон рендеринга в системе. |
| [RenderWindow_GetCursorPosition](../help/topics/RenderWindow_GetCursorPosition.md) | `RenderWindow_GetCursorPosition(HANDLE, &FLOAT, &FLOAT)` | 1295 | Функция для получения координат курсора во внешнем окне рендеринга. |
| [RenderWindow_GetKeyboardButtonPressed](../help/topics/RenderWindow_GetKeyboardButtonPressed.md) | `FLOAT RenderWindow_GetKeyboardButtonPressed(HANDLE, FLOAT)` | 1298 | Функция для получения состояния кнопки клавиатуры для внешнего окна рендеринга. |
| [RenderWindow_GetMouseButtonPressed](../help/topics/RenderWindow_GetMouseButtonPressed.md) | `FLOAT RenderWindow_GetMouseButtonPressed(HANDLE, FLOAT)` | 1297 | Функция для получения состояния кнопки мыши для внешнего окна рендеринга. |
| [RenderWindow_GetPosition](../help/topics/RenderWindow_GetPosition.md) | `RenderWindow_GetPosition(HANDLE, &FLOAT, &FLOAT)` | 1312 | Функция для получения координат верхнего левого угла окна рендеринга. |
| [RenderWindow_GetPrimary](../help/topics/RenderWindow_GetPrimary.md) | `HANDLE RenderWindow_GetPrimary()` | 1023 | Функция получения первичного (основного) окна рендеринга, которое создается первым. |
| [RenderWindow_GetSize](../help/topics/RenderWindow_GetSize.md) | `RenderWindow_GetSize(HANDLE, &FLOAT, &FLOAT)` | 1313 | Функция для получения размера окна рендеринга. |
| [RenderWindow_GetViewport](../help/topics/RenderWindow_GetViewport.md) | `HANDLE RenderWindow_GetViewport(HANDLE, FLOAT)` | 1300 | Функция для получения объекта класса Viewport по индексу. |
| [RenderWindow_GetViewportCount](../help/topics/RenderWindow_GetViewportCount.md) | `FLOAT RenderWindow_GetViewportCount(HANDLE)` | 1299 | Функция для получения количества объектов класса Viewport подключенных к окну рендеринга. |
| [RenderWindow_GetWheelPosition](../help/topics/RenderWindow_GetWheelPosition.md) | `FLOAT RenderWindow_GetWheelPosition(HANDLE)` | 1303 | Функция для получения координаты задаваемой колесом мыши для внешнего окна рендеринга. |
| [RenderWindow_SetPosition](../help/topics/RenderWindow_SetPosition.md) | `RenderWindow_SetPosition(HANDLE, FLOAT, FLOAT)` | 1025 | Функция используется для установки координат окна рендеринга в пространстве окна системы Stratum. |
| [RenderWindow_SetSize](../help/topics/RenderWindow_SetSize.md) | `RenderWindow_SetSize(HANDLE, FLOAT, FLOAT)` | 1026 | Функция используется для установки размера окна рендеринга. |
| [RenderWindow_ToggleFullscreen](../help/topics/RenderWindow_ToggleFullscreen.md) | `FLOAT RenderWindow_ToggleFullscreen(HANDLE)` | 1076 | Функция для переключения окна рендеринга в полноэкранный режим и обратно в оконный. |
| [Root_AddResourceLocation](../help/topics/Root_AddResourceLocation.md) | `Root_AddResourceLocation(STRING, STRING, STRING, FLOAT)` | 1036 | Функция используется для добавления источников ресурсов, которые указаны в конфигурационном файле. |
| [Root_AddResourceLocationFromConfigFile](../help/topics/Root_AddResourceLocationFromConfigFile.md) | `Root_AddResourceLocationFromConfigFile(STRING)` | 1035 | Функция используется для добавления источников ресурсов, которые указаны в конфигурационном файле. |
| [Root_Create](../help/topics/Root_Create.md) | `Root_Create(STRING, STRING, STRING)` | 1027 | Функция используется для создания основного объекта системы визуализации, она должна вызываться в первую очередь при запуске проекта. |
| [Root_Destroy](../help/topics/Root_Destroy.md) | `Root_Destroy()` | 1028 | Функция для удаления основного объекта системы визуализации. Вызывается системой при остановке модели, поэтому при нормальном использовании её вызов не нужен. |
| [Root_GetTime](../help/topics/Root_GetTime.md) | `FLOAT Root_GetTime()` | 1038 | Данная функция используется для получения текущего времени работы системы визуализации, время измеряется в секундах. |
| [Root_Initialise](../help/topics/Root_Initialise.md) | `Root_Initialise()` | 1029 | Функция для инициализации системы визуализации. До её вызова необходимо сконфигурировать систему с помощью загрузки параметров из файла (Root_RestoreConfig()) или диалога настроек (Root_ShowConfigDialog()). |
| [Root_InitialiseAllResourceGroups](../help/topics/Root_InitialiseAllResourceGroups.md) | `Root_InitialiseAllResourceGroups()` | 1037 | Данная функция запускает процесс инициализаии источников ресурсов, то есть все найденные в источниках ресурсы будут проиндексированы. Перед тем как использовать ресурсы следует вызвать данную функцию. Перед вызовом данной функции необходимо указать источники ресурсов. |
| [Root_IsInitialised](../help/topics/Root_IsInitialised.md) | `FLOAT Root_IsInitialised()` | 1030 | Функция для определения состояния инициализации системы визуализации. |
| [Root_RenderOneFrame](../help/topics/Root_RenderOneFrame.md) | `FLOAT Root_RenderOneFrame()` | 1034 | Данная функция запускает процесс визуализации сцен  в окна рендеринга. Происходит отрисовка только одного кадра. |
| [Root_RestoreConfig](../help/topics/Root_RestoreConfig.md) | `FLOAT Root_RestoreConfig()` | 1031 | Функция для загрузки параметров системы рендеринга из файла, имя которого указано при создании объекта класса Root. |
| [Root_SaveConfig](../help/topics/Root_SaveConfig.md) | `Root_SaveConfig()` | 1032 | Функция для сохранения параметров системы рендеринга в файл, имя которого указано при создании объекта класса Root. |
| [Root_ShowConfigDialog](../help/topics/Root_ShowConfigDialog.md) | `FLOAT Root_ShowConfigDialog()` | 1033 | Данная функция отображает на экране диалог, где пользователь может установить параметры системы рендеринга. |
| [Scene_Clear](../help/topics/Scene_Clear.md) | `Scene_Clear(HANDLE)` | 1053 | Функция используется для удаления всех объектов, включенных в иерархию данной сцены. |
| [Scene_Create](../help/topics/Scene_Create.md) | `HANDLE Scene_Create(STRING)` | 1039 | Функция используется для создания сцены. |
| [Scene_Destroy](../help/topics/Scene_Destroy.md) | `Scene_Destroy(HANDLE)` | 1040 | Функция используется для удаления сцены. |
| [Scene_GetCamera](../help/topics/Scene_GetCamera.md) | `HANDLE Scene_GetCamera(HANDLE, STRING)` | 1048 | Функция используется для получения находящейся на сцене камеры по её имени. |
| [Scene_GetEntity](../help/topics/Scene_GetEntity.md) | `HANDLE Scene_GetEntity(HANDLE, STRING)` | 1049 | Функция используется для получения находящейся на сцене трехмерной модели по её имени. |
| [Scene_GetLight](../help/topics/Scene_GetLight.md) | `HANDLE Scene_GetLight(HANDLE, STRING)` | 1050 | Функция используется для получения находящегося на сцене источника света по его имени. |
| [Scene_GetParticleSystem](../help/topics/Scene_GetParticleSystem.md) | `HANDLE Scene_GetParticleSystem(HANDLE, STRING)` | 1051 | Функция используется для получения находящейся на сцене системы частиц по её имени. |
| [Scene_GetRootSceneNode](../help/topics/Scene_GetRootSceneNode.md) | `HANDLE Scene_GetRootSceneNode(HANDLE)` | 1042 | Функция используется для получения корневого узла, из которого будет строится вся иерархия сцены. |
| [Scene_GetSceneNode](../help/topics/Scene_GetSceneNode.md) | `HANDLE Scene_GetSceneNode(HANDLE, STRING)` | 1052 | Функция используется для получения узла сцены по его имени. |
| [Scene_GetShadowTechnique](../help/topics/Scene_GetShadowTechnique.md) | `FLOAT Scene_GetShadowTechnique(HANDLE)` | 1044 | Функция используется для получения режима визуализации теней. |
| [Scene_Load](../help/topics/Scene_Load.md) | `Scene_Load(HANDLE, STRING)` | 1054 | Функция используется для загрузки сцены из файла. |
| [Scene_SetAmbientLight](../help/topics/9II5_D.md) | `Scene_SetAmbientLight(HANDLE, FLOAT, FLOAT, FLOAT)` | 1046 | Функция используется для установки цвета рассеянного фонового освещения сцены. |
| [Scene_SetFog](../help/topics/Scene_SetFog.md) | `Scene_SetFog(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 1045 | Функция используется для установки режима тумана. |
| [Scene_SetShadowTechnique](../help/topics/Scene_SetShadowTechnique.md) | `Scene_SetShadowTechnique(HANDLE, FLOAT)` | 1043 | Функция используется для установки режима визуализации теней. |
| [Scene_SetSkyBox](../help/topics/Scene_SetSkyBox.md) | `Scene_SetSkyBox(HANDLE, FLOAT, STRING, FLOAT)` | 1047 | Функция используется для настройки объекта SkyBox. |
| [Scene_SetWorldGeometry](../help/topics/Scene_SetWorldGeometry.md) | `Scene_SetWorldGeometry(HANDLE, STRING)` | 1041 | Функция используется для загрузки статической фоновой трехмерной модели. Эта функция поддерживается каждым типом сцены по-своему. Для BSPSceneManager будет загружаться BSP уровень, а для TerrainSceneManager будет загружаться ландшафт. |
| [SceneNode_AttachObject](../help/topics/SceneNode_AttachObject.md) | `SceneNode_AttachObject(HANDLE, HANDLE)` | 1058 | Функция используется для подключения (включения в иерархию сцены) перемещаемого объекта к узлу сцены. |
| [SceneNode_Create](../help/topics/SceneNode_Create.md) | `HANDLE SceneNode_Create(HANDLE, STRING)` | 1055 | Функция используется для создания узла сцены. |
| [SceneNode_Destroy](../help/topics/SceneNode_Destroy.md) | `SceneNode_Destroy(HANDLE)` | 1056 | Функция используется для удаления узла сцены. |
| [SceneNode_DetachObject](../help/topics/SceneNode_DetachObject.md) | `SceneNode_DetachObject(HANDLE, HANDLE)` | 1059 | Функция используется для отключения (исключения из иерархии сцены) перемещаемого объекта от узла сцены. При этом отключаемый объект должен быть одним из дочерних объектов узла сцены. |
| [SceneNode_GetNumObjects](../help/topics/SceneNode_GetNumObjects.md) | `FLOAT SceneNode_GetNumObjects(HANDLE)` | 1214 | Функция для получения количества подключенных к узлу сцены перемещаемых объектов. |
| [SceneNode_GetObject](../help/topics/SceneNode_GetObject.md) | `HANDLE SceneNode_GetObject(HANDLE, FLOAT)` | 1215 | Функция для получения подключенного к узлу перемещаемого объекта по его индексу. |
| [SceneNode_SetAutoTracking](../help/topics/SceneNode_SetAutoTracking.md) | `SceneNode_SetAutoTracking(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 1057 | Функция используется для автоматической ориентации узла сцены в направлении другого узла сцены. |
| [SceneNode_SetVisible](../help/topics/SceneNode_SetVisible.md) | `SceneNode_SetVisible(HANDLE, FLOAT, FLOAT)` | 1216 | Функция для установки видимости подключенных к узлу перемещаемых объектов. |
| [StringInterface_GetParameter](../help/topics/StringInterface_GetParameter.md) | `StringInterface_GetParameter(HANDLE, STRING, STRING)` | 1288 | Получение параметра объекта с помощью текстового интерфейса. |
| [StringInterface_GetParameterCount](../help/topics/StringInterface_GetParameterCount.md) | `FLOAT StringInterface_GetParameterCount(HANDLE)` | 1289 | Получение количество параметров текстового интерфейса. |
| [StringInterface_GetParameterDescription](../help/topics/StringInterface_GetParameterDescription.md) | `StringInterface_GetParameterDescription(HANDLE, FLOAT, STRING)` | 1292 | Получение описания параметра текстового интерфейса по индексу. |
| [StringInterface_GetParameterName](../help/topics/StringInterface_GetParameterName.md) | `StringInterface_GetParameterName(HANDLE, FLOAT, STRING)` | 1291 | Получение имени параметра текстового интерфейса по индексу. |
| [StringInterface_GetParameterType](../help/topics/StringInterface_GetParameterType.md) | `FLOAT StringInterface_GetParameterType(HANDLE, FLOAT)` | 1290 | Получение типа параметра по индексу. |
| [StringInterface_SetParameter](../help/topics/StringInterface_SetParameter.md) | `StringInterface_SetParameter(HANDLE, STRING, STRING)` | 1287 | Установка параметра объекта с помощью текстового интерфейса. |
| [Technique_Create](../help/topics/Technique_Create.md) | `HANDLE Technique_Create(HANDLE, STRING)` | 1060 | Функция используется для создания техники материала. |
| [Technique_GetPassByIndex](../help/topics/Technique_GetPassByIndex.md) | `HANDLE Technique_GetPassByIndex(HANDLE, FLOAT)` | 1062 | Функция используется для получения итерации материала по её индексу. Итерации хранятся в техниках. |
| [Technique_GetPassByName](../help/topics/Technique_GetPassByName.md) | `HANDLE Technique_GetPassByName(HANDLE, STRING)` | 1061 | Функция используется для получения итерации материала по её имени. Итерации хранятся в техниках. |
| [TextureUnitState_Create](../help/topics/TextureUnitState_Create.md) | `HANDLE TextureUnitState_Create(HANDLE, STRING)` | 1063 | Функция используется для создания текстурного слота. |
| [TextureUnitState_SetTexture](../help/topics/TextureUnitState_SetTexture.md) | `TextureUnitState_SetTexture(HANDLE, STRING, FLOAT)` | 1064 | Функция используется для установки текстуры. |
| [Viewport_AddCompositor](../help/topics/Viewport_AddCompositor.md) | `HANDLE Viewport_AddCompositor(HANDLE, STRING, FLOAT)` | 1200 | Функция для создания и добавления композитора. |
| [Viewport_Create](../help/topics/Viewport_Create.md) | `HANDLE Viewport_Create(HANDLE, HANDLE, FLOAT)` | 1066 | Функция используется для создания области рендеринга. |
| [Viewport_Destroy](../help/topics/Viewport_Destroy.md) | `Viewport_Destroy(HANDLE)` | 1067 | Функция используется для удаления области рендеринга. |
| [Viewport_GetBackgroundColour](../help/topics/Viewport_GetBackgroundColour.md) | `Viewport_GetBackgroundColour(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1310 | Функция для получения цвета фона у вьюпорта. |
| [Viewport_GetCamera](../help/topics/Viewport_GetCamera.md) | `HANDLE Viewport_GetCamera(HANDLE)` | 1069 | Функция используется для получения используемой камеры в области рендеринга. |
| [Viewport_GetCompositor](../help/topics/Viewport_GetCompositor.md) | `HANDLE Viewport_GetCompositor(HANDLE, FLOAT)` | 1203 | Функция для получения композитора по индексу. |
| [Viewport_GetDimensions](../help/topics/Viewport_GetDimensions.md) | `Viewport_GetDimensions(HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1311 | Функция для получения относительных координат и размера вьюпорта. |
| [Viewport_GetNumCompositors](../help/topics/Viewport_GetNumCompositors.md) | `FLOAT Viewport_GetNumCompositors(HANDLE)` | 1202 | Функция для получения количества зарегистрированных композиторов. |
| [Viewport_GetOverlaysEnabled](../help/topics/Viewport_GetOverlaysEnabled.md) | `FLOAT Viewport_GetOverlaysEnabled(HANDLE)` | 1073 | Функция используется для получения флага отображения оверлеев. |
| [Viewport_GetRay](../help/topics/Viewport_GetRay.md) | `Viewport_GetRay(HANDLE, FLOAT, FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 1074 | Функция используется для получения луча в пространстве сцены по заданным координатам в области рендеринга. |
| [Viewport_RemoveCompositor](../help/topics/Viewport_RemoveCompositor.md) | `Viewport_RemoveCompositor(HANDLE, HANDLE)` | 1201 | Функция для удаления композитора. |
| [Viewport_SetBackgroundColour](../help/topics/Viewport_SetBackgroundColour.md) | `Viewport_SetBackgroundColour(HANDLE, FLOAT, FLOAT, FLOAT)` | 1070 | Функция используется для установки цвета фона для области рендеринга. |
| [Viewport_SetCamera](../help/topics/Viewport_SetCamera.md) | `Viewport_SetCamera(HANDLE, HANDLE)` | 1068 | Функция используется для установки новой камеры для области рендеринга. |
| [Viewport_SetOverlaysEnabled](../help/topics/Viewport_SetDimensions.md) | `Viewport_SetOverlaysEnabled(HANDLE, FLOAT)` | 1072 | Функция используется для установки флага отображения оверлеев в области рендеринга. |

## Функции работы с 2D-графикой (150)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [AddGroupItem2d](../help/topics/AddGroupItem2d.md) | `FLOAT AddGroupItem2d(HANDLE, HANDLE, HANDLE)` | 366 | Функция добавляет в указанную группу новый графический объект. |
| [AddPoint2d](../help/topics/AddPoint2d.md) | `FLOAT AddPoint2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 301 | Функция добавляет новую точку к множеству точек, из которых состоит указанная линия. |
| [AddText2d](../help/topics/AddText2d.md) | `FLOAT AddText2d(HANDLE, HANDLE, FLOAT, HANDLE, HANDLE, COLORREF, COLORREF)`<br>`FLOAT AddText2d(HANDLE, HANDLE, HANDLE, HANDLE, COLORREF, COLORREF)` | 1111, 1113 | Функция позволяет вставить элемент в логический текст |
| [CopyToClipboard2d](../help/topics/CopyToClipboard2d.md) | `FLOAT CopyToClipboard2d(HANDLE, HANDLE)` | 376 | Функция помещает заданный объект в clipboard. |
| [CreateBitmap2d](../help/topics/CreateBitmap2d.md) | `HANDLE CreateBitmap2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 339 | Функция создает двухмерный объект - битовую карту. |
| [CreateBrush2d](../help/topics/CreateBrush2d.md) | `HANDLE CreateBrush2d(HANDLE, FLOAT, FLOAT, COLORREF, HANDLE, FLOAT)` | 333 | Функция создает логическую кисть. |
| [CreateDIB2d](../help/topics/CreateDIB2d.md) | `HANDLE CreateDIB2d(HANDLE, STRING)` | 335 | Функция создает битовую карту из файла. Поддерживаются форматы gif, pcx, tga, bmp, rle, jpg. |
| [CreateDoubleBitmap2d](../help/topics/CreateDoubleBitmap2d.md) | `HANDLE CreateDoubleBitmap2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 340 | Функция создает двухмерный объект - двойную битовую карту (см. Графика). |
| [CreateDoubleDIB2d](../help/topics/CreateDoubleDIB2d.md) | `HANDLE CreateDoubleDIB2d(HANDLE, STRING)` | 336 | Функция позволяет создать [двойную битовую карту](DoubleDib.md) из файла. |
| [CreateFont2d](../help/topics/CreateFont2d.md) | `HANDLE CreateFont2d(HANDLE, STRING, FLOAT, FLOAT)` | 360 | Функция создает инструмент-шрифт, который потом можно использовать при создании объекта-текста |
| [CreateGroup2d](../help/topics/CreateGroup2d.md) | `HANDLE CreateGroup2d(HANDLE, [HANDLE])` | 367 | Функция создает группу графических объектов. Имеет переменное число аргументов для задания дескрипторов объектов, объединяющихся в группу. |
| [CreateObjectFromFile2d](../help/topics/CreateObjectFromFile2D.md) | `HANDLE CreateObjectFromFile2d(HANDLE, STRING, FLOAT, FLOAT, FLOAT)` | 104 | Функция создает в указанном пространстве объект, загрузив его из VDR файла. |
| [CreatePen2d](../help/topics/CreatePen2d.md) | `HANDLE CreatePen2d(HANDLE, FLOAT, FLOAT, COLORREF, FLOAT)` | 313 | Функция позволяет создать карандаш. |
| [CreatePolyLine2d](../help/topics/CreatePolyLine2d.md) | `HANDLE CreatePolyLine2d(HANDLE, HANDLE, HANDLE, [FLOAT, FLOAT])` | 485 | Функция создает линию. Функция имеет переменное число параметров X и Y, что позволяет |
| [CreateRasterText2d](../help/topics/CreateRasterText2d.md) | `HANDLE CreateRasterText2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 363 | Функция создает графический объект - текст. |
| [CreateRDIB2d](../help/topics/CreateRDIB2d.md) | `HANDLE CreateRDIB2d(HANDLE, STRING)` | 337 | Функция создает битовую карту из файла, как разделяемый ресурс (ссылка на файл). Поддерживаются форматы pcx, bmp, ico, jpg, tif, gif и tga. |
| [CreateRDoubleDIB2d](../help/topics/CreateRDoubleDIB2d.md) | `HANDLE CreateRDoubleDIB2d(HANDLE, STRING)` | 338 | Функция позволяет создать [двойную битовую карту](DoubleDib.md) из файла как разделяемый ресурс. Поддерживается форматы pcx, bmp, ico, jpg, tif, gif и tga. |
| [CreateString2d](../help/topics/CreateString2d.md) | `HANDLE CreateString2d(HANDLE, STRING)` | 361 | Функция создает логическую строку текста. |
| [CreateText2d](../help/topics/CreateText2d.md) | `HANDLE CreateText2d(HANDLE, HANDLE, HANDLE, COLORREF, COLORREF)` | 362 | Функция создает логический текст. |
| [CreateWindowEx](../help/topics/CreateWindowEx.md) | `HANDLE CreateWindowEx(STRING, STRING, STRING, FLOAT, FLOAT, FLOAT, FLOAT, STRING)` | 421 |  |
| [DeleteGroup2d](../help/topics/DeleteGroup2d.md) | `FLOAT DeleteGroup2d(HANDLE, HANDLE)` | 368 | Функция разгруппировывает группу. Если группа сама является членом группы, то она из нее удаляется. |
| [DeleteObject2d](../help/topics/DeleteObject2d.md) | `FLOAT DeleteObject2d(HANDLE, HANDLE)` | 323 | Функция позволяет удалить графический объект. |
| [DeleteTool2d](../help/topics/DeleteTool2d.md) | `FLOAT DeleteTool2d(HANDLE, FLOAT, HANDLE)` | 322 | Функция удаляет различные графические инструменты из графического пространства. |
| [DelGroupItem2d](../help/topics/DelGroupItem2d.md) | `FLOAT DelGroupItem2d(HANDLE, HANDLE, HANDLE)` | 369 | Функция удаляет объект из группы. |
| [DelPoint2d](../help/topics/Delpoint2d.md) | `FLOAT DelPoint2d(HANDLE, HANDLE, FLOAT)` | 302 | Функция удаляет одну из точек линии. |
| [EmptySpace2d](../help/topics/EmptySpace2d.md) | `FLOAT EmptySpace2d(HANDLE)` | 482 | Функция очищает двухмерное пространство от всех существующих в нем объектов. |
| [GetActualHeight2d](../help/topics/GetActualWidth2d.md) | `FLOAT GetActualHeight2d(HANDLE, HANDLE)` | 243 | Функция позволяет получить актуальную ширину двухмерного объекта. |
| [GetActualSize2d](../help/topics/GetActualSize2d.md) | `FLOAT GetActualSize2d(HANDLE, HANDLE, &FLOAT, &FLOAT)` | 737 | Функция позволяет получить актуальные размеры двухмерного объекта. |
| [GetBitmapSrcRect2d](../help/topics/GetBitmapSrcRect2d.md) | `FLOAT GetBitmapSrcRect2d(HANDLE, HANDLE, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 754 | Функция позволяет получить фрагмент битовой карты, который отображается на экране. |
| [GetBkBrush2d](../help/topics/GetBkBrush2d.md) | `HANDLE GetBkBrush2d(HANDLE)` | 739 | Функция возвращает дескриптор фоновой кисти. |
| [GetBottomObject2d](../help/topics/GetBottomObject2d.md) | `HANDLE GetBottomObject2d(HANDLE)` | 347 | Функция определяет дескриптор объекта, который находится позади всех других (по Z - порядку). |
| [GetBrushColor2d](../help/topics/GetBrushColor2d.md) | `COLORREF GetBrushColor2d(HANDLE, HANDLE)` | 401 | Функция определяет текущий цвет кисти. |
| [GetBrushDib2d](../help/topics/GetBrushDib2d.md) | `HANDLE GetBrushDib2d(HANDLE, HANDLE)` | 403 | Функция определяет дескриптор битовой карты, использованной в заливке. |
| [GetBrushHatch2d](../help/topics/GetBrushHatch2d.md) | `FLOAT GetBrushHatch2d(HANDLE, HANDLE)` | 405 | Функция определяет текущий тип штриховки используемой в кисти (см. [Типы штриховки](Hatch.md)). |
| [GetBrushObject2d](../help/topics/GetBrushObject2d.md) | `HANDLE GetBrushObject2d(HANDLE, HANDLE)` | 303 | Функция возвращает дескриптор кисти, которая используется в указанном графическом объекте. |
| [GetBrushRop2d](../help/topics/GetBrushRop2d.md) | `FLOAT GetBrushRop2d(HANDLE, HANDLE)` | 402 | Функция определяет ткущую логическую операцию, которая используется в кисти (см. [Логические операции с кистью](ROP.md)). |
| [GetBrushStyle2d](../help/topics/GetBrushStyle2d.md) | `FLOAT GetBrushStyle2d(HANDLE, HANDLE)` | 404 | Функция возвращает стиль, применяемый в указанной кисти (см. [Стили кисти](BrushStyle.md)). |
| [GetBValue](../help/topics/GetBValue.md) | `FLOAT GetBValue(COLORREF)` | 611 | Функция позволяет получить конкретный компонент из цвета. Является обратной для функции RGB |
| [GetCurrentObject2d](../help/topics/GetCurrentObject2d.md) | `HANDLE GetCurrentObject2d(HANDLE)` | 667 | Функция позволяет определить текущий двухмерный объект. |
| [GetDDibObject2d](../help/topics/GetDDibObject2d.md) | `HANDLE GetDDibObject2d(HANDLE, HANDLE)` | 746 | Функция позволяет определить дескриптор двойной битовой карты, используемой данным двухмерным объектом. |
| [GetDibObject2d](../help/topics/GetDibObject2d.md) | `HANDLE GetDibObject2d(HANDLE, HANDLE)` | 744 | Функция позволяет определить дескриптор битовой карты, используемой данным двухмерным объектом. |
| [GetDibPixel2d](../help/topics/GetDibPixel2d.md) | `COLORREF GetDibPixel2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 474 | Функция для получения цвета пиксела битовой карты в заданных координата. |
| [GetFontList](../help/topics/GetFontList.md) | `HANDLE GetFontList()` | 872 | Функция позволяет получить список имен шрифтов TrueType, доступных в системе |
| [GetFontName2d](../help/topics/GetFontName2d.md) | `STRING GetFontName2d(HANDLE, HANDLE)` | 846 | Функция позволяет получить имя шрифта |
| [GetFontSize2d](../help/topics/GetFontSize2d.md) | `FLOAT GetFontSize2d(HANDLE, HANDLE)` | 847 | Функция позволяет получить размер шрифта в пунктах (логических единицах) |
| [GetFontStyle2d](../help/topics/GetFontStyle2d.md) | `FLOAT GetFontStyle2d(HANDLE, HANDLE)` | 849 | Функция позволяет получить стиль шрифта |
| [GetGroupItem2d](../help/topics/GetGroupItem2d.md) | `HANDLE GetGroupItem2d(HANDLE, HANDLE, FLOAT)` | 370 | Функция возвращает дескриптор одного из элементов группы по его номеру. |
| [GetGroupItemsNum2d](../help/topics/GetGroupItemsNum2d.md) | `FLOAT GetGroupItemsNum2d(HANDLE, HANDLE)` | 372 | Функция возвращает число объектов в группе. |
| [GetGValue](../help/topics/GetGValue.md) | `FLOAT GetGValue(COLORREF)` | 610 | Функция позволяет получить конкретный компонент из цвета. Является обратной для функции RGB |
| [GetLastPrimary2d](../help/topics/GetLastPrimary2d.md) | `HANDLE GetLastPrimary2d()` | 381 | Функция возвращает дескриптор первичного объекта после вызова функции GetObjectFromPoint2d. |
| [GetLowerObject2d](../help/topics/GetLowerObject2d.md) | `HANDLE GetLowerObject2d(HANDLE, HANDLE)` | 350 | Функция возвращает дескриптор объекта, который находится позади заданного (по Z - ордеру). |
| [GetNextObject2d](../help/topics/GetNextObject2d.md) | `HANDLE GetNextObject2d(HANDLE, HANDLE)` | 614 | Функция возвращает дескриптор следующего по порядку объекта за указанным. Если объект последний возвращается ноль. |
| [GetNextTool2d](../help/topics/GetNextTool2d.md) | `HANDLE GetNextTool2d(HANDLE, FLOAT, HANDLE)` | 613 | Функция возвращает дескриптор следующего по порядку инструмента за указанным. Если инструмент последний возвращается ноль. |
| [GetObject2dByName](../help/topics/GetObject2dByName.md) | `HANDLE GetObject2dByName(HANDLE, HANDLE, STRING)` | 224 | Функция возвращает дескриптор графического объекта по его имени. |
| [GetObjectAlpha2d](../help/topics/GetObjectAlpha2d.md) | `FLOAT GetObjectAlpha2d(HANDLE, HANDLE)` | 1122 | Функция для определения прозрачности графического объекта |
| [GetObjectAngle2d](../help/topics/GetObjectAngle2d.md) | `FLOAT GetObjectAngle2d(HANDLE, HANDLE)` | 327 | Функция возвращает угол в радианах, под которым отображается двухмерный объект. |
| [GetObjectAttribute2d](../help/topics/GetObjectAttribute2d.md) | `FLOAT GetObjectAttribute2d(HANDLE, HANDLE)` | 359 | Возвращает атрибуты объекта. |
| [GetObjectFromPoint2d](../help/topics/GetObjectFromPoint2d.md) | `HANDLE GetObjectFromPoint2d(HANDLE, FLOAT, FLOAT)` | 380 | Функция возвращает дескриптор двухмерного объекта, располагающегося по указанным координатам. |
| [GetObjectFromPoint2dEx](../help/topics/GetObjectFromPoint2dEx.md) | `HANDLE GetObjectFromPoint2dEx(HANDLE, FLOAT, FLOAT, FLOAT)` | 440 | Функция возвращает дескриптор двухмерного объекта, располагающегося по указанным координатам, в указанном слое. |
| [GetObjectFromZOrder2d](../help/topics/GetObjectFromZOrder2d.md) | `HANDLE GetObjectFromZOrder2d(HANDLE, FLOAT)` | 349 | Функция возвращает дескриптор объекта, который находится в заданном месте Z списка. |
| [GetObjectHeight2d](../help/topics/GetObjectHeight2d.md) | `FLOAT GetObjectHeight2d(HANDLE, HANDLE)` | 329 | Функция возвращает размер объекта. |
| [GetObjectName2d](../help/topics/GetObjectName2d.md) | `STRING GetObjectName2d(HANDLE, HANDLE)` | 378 | Функция возвращает имя двухмерного объекта по его дескриптору. |
| [GetObjectOrg2dx](../help/topics/GetObjectOrg2dx.md) | `FLOAT GetObjectOrg2dx(HANDLE, HANDLE)` | 324 | Функция возвращает координату объекта в пространстве. |
| [GetObjectOrg2dy](../help/topics/GetObjectOrg2dy.md) | `FLOAT GetObjectOrg2dy(HANDLE, HANDLE)` | 325 | Функция возвращает координату объекта в пространстве. |
| [GetObjectParent2d](../help/topics/GetObjectParent2d.md) | `HANDLE GetObjectParent2d(HANDLE, HANDLE)` | 326 | Функция позволяет вернуть дескриптор группы, в которую входит указанный объект. |
| [GetObjectType2d](../help/topics/GetObjectType.md) | `FLOAT GetObjectType2d(HANDLE, HANDLE)` | 330 | Функция возвращает номер, соответствующий [типу графического объекта](objects_type.md). |
| [GetObjectWidth2d](../help/topics/GetObjectWidth2d.md) | `FLOAT GetObjectWidth2d(HANDLE, HANDLE)` | 328 | Функция возвращает размер объекта. |
| [GetPenColor2d](../help/topics/GetPenColor2d.md) | `COLORREF GetPenColor2d(HANDLE, HANDLE)` | 314 | Функция возвращает цвет карандаша. |
| [GetPenObject2d](../help/topics/GetPenObject2d.md) | `HANDLE GetPenObject2d(HANDLE, HANDLE)` | 304 | Функция возвращает дескриптор карандаша, который используется при рисовании линии. |
| [GetPenRop2d](../help/topics/GetPenrop2d.md) | `FLOAT GetPenRop2d(HANDLE, HANDLE)` | 315 | Функция возвращает логическую операцию при рисовании карандашом. |
| [GetPenStyle2d](../help/topics/GetPenStyle2d.md) | `FLOAT GetPenStyle2d(HANDLE, HANDLE)` | 316 | Функция возвращает стиль карандаша. |
| [GetPenWidth2d](../help/topics/GetPenWidth2d.md) | `FLOAT GetPenWidth2d(HANDLE, HANDLE)` | 317 | Функция возвращает толщину карандаша. |
| [GetRGNCreateMode](../help/topics/GetRGNCreateMode.md) | `FLOAT GetRGNCreateMode(HANDLE, HANDLE)` | 305 | Функция возвращает режим заполнения региона. |
| [GetRValue](../help/topics/GetRValue.md) | `FLOAT GetRValue(COLORREF)` | 609 | Функция позволяет получить конкретный компонент из цвета. Является обратной для функции RGB |
| [GetScaleSpace2d](../help/topics/GetScaleSpace2d.md) | `FLOAT GetScaleSpace2d(HANDLE)` | 345 | Функция возвращает масштаб отображения двухмерного пространства. |
| [GetSpaceLayers2d](../help/topics/GetSpaceLayers2d.md) | `FLOAT GetSpaceLayers2d(HANDLE)` | 488 | Функция возвращает состояние видимости слоев в двухмерном пространстве. |
| [GetSpaceOrg2dx](../help/topics/GetSpaceOrg2dX.md) | `FLOAT GetSpaceOrg2dx(HANDLE)` | 342 | Функция возвращает логическую точку пространства, которая соответствует левому верхнему углу окна. |
| [GetSpaceOrg2dy](../help/topics/GetSpaceOrg2dY.md) | `FLOAT GetSpaceOrg2dy(HANDLE)` | 341 | Функция возвращает логическую точку пространства, которая соответствует левому верхнему углу окна. |
| [GetString2d](../help/topics/GetString2d.md) | `STRING GetString2d(HANDLE, HANDLE)` | 397 | Функция позволяет получить текст инструмента - графической строки. |
| [GetTextBkColor](../help/topics/GetTextBkColor2d.md) | `COLORREF GetTextBkColor(HANDLE HSpace2d, HANDLE HText)`<br>`COLORREF GetTextBkColor(HANDLE HSpace2d, HANDLE HText, FLOAT Index)` |  | Функция позволяет получить цвет фона текста. |
| [GetTextCount2d](../help/topics/GetTextCount2d.md) | `FLOAT GetTextCount2d(HANDLE, HANDLE)` | 848 | Функция позволяет получить количество элементов в логическом тексте |
| [GetTextFgColor](../help/topics/GetTextFgColor2d.md) | `COLORREF GetTextFgColor(HANDLE HSpace2d, HANDLE HText)`<br>`COLORREF GetTextFgColor(HANDLE HSpace2d, HANDLE HText, FLOAT Index)` |  | Функция позволяет получить цвет текста |
| [GetTextFont2d](../help/topics/GetTextFont2d.md) | `HANDLE GetTextFont2d(HANDLE, HANDLE)`<br>`HANDLE GetTextFont2d(HANDLE, HANDLE, FLOAT)` | 495, 854 | Функция позволяет получить дескриптор шрифта, используемого в элементе текста |
| [GetTextObject2d](../help/topics/GetTextObject2d.md) | `HANDLE GetTextObject2d(HANDLE, HANDLE)` | 493 | Функция позволяет получить логический текст, который используется данным двухмерным объектом. |
| [GetTextString2d](../help/topics/GetTextString2d.md) | `HANDLE GetTextString2d(HANDLE, HANDLE)`<br>`HANDLE GetTextString2d(HANDLE, HANDLE, FLOAT)` | 494, 853 | Функция позволяет получить дескриптор строки текста. |
| [GetToolRef2d](../help/topics/GetToolRef2d.md) | `FLOAT GetToolRef2d(HANDLE, FLOAT, HANDLE)` | 612 | Функция позволяет получить число ссылок на инструмент. |
| [GetTopObject2d](../help/topics/GetTopObject2d.md) | `HANDLE GetTopObject2d(HANDLE)` | 351 | Функция возвращает дескриптор объекта, который находится поверх всех других (по Z-порядку). |
| [GetUpperObject2d](../help/topics/GetUpperObject2d.md) | `HANDLE GetUpperObject2d(HANDLE, HANDLE)` | 348 | Функция возвращает дескриптор объекта, который находится поверх заданного (по Z-ордеру). |
| [GetVectorNumPoints2d](../help/topics/GetVectorNumPoints2d.md) | `FLOAT GetVectorNumPoints2d(HANDLE, HANDLE)` | 306 | Функция возвращает число точек в линии. |
| [GetVectorPoint2dx](../help/topics/GetVectorPoint2dx.md) | `FLOAT GetVectorPoint2dx(HANDLE, HANDLE, FLOAT)` | 307 | Функция возвращает координаты точки в линии. |
| [GetVectorPoint2dy](../help/topics/GetVectorPoint2dy.md) | `FLOAT GetVectorPoint2dy(HANDLE, HANDLE, FLOAT)` | 308 | Функция возвращает координаты точки в линии. |
| [GetZOrder2d](../help/topics/GetZOrder2d.md) | `FLOAT GetZOrder2d(HANDLE, HANDLE)` | 352 | Функция возвращает место в Z списке, которое занимает двухмерный объект. |
| [HideObject2d](../help/topics/HideObject2d.md) | `HideObject2d(HANDLE, HANDLE)` | 384 | Функция отключает видимость объекта. Для включения видимости используется [ShowObject](class_ShowObject.md). |
| [IsGroupContainObject2d](../help/topics/IsGroupContainObject2d.md) | `FLOAT IsGroupContainObject2d(HANDLE, HANDLE, HANDLE)` | 373 | Функция определяет - содержит ли группа указанный объект. |
| [IsObjectsIntersect2d](../help/topics/IsObjectsIntersect2d.md) | `FLOAT IsObjectsIntersect2d(HANDLE, HANDLE, HANDLE, FLOAT)` | 298 | Функция позволяет определить факт пересечения двух графических объектов. |
| [LoadSpacewindow](../help/topics/LoadSpaceWindow.md) | `HANDLE LoadSpacewindow(STRING, STRING, STRING)` | 200 | Функция создает окно с заданным именем и заполняет его графическими объектами из файла. |
| [LockObject2d](../help/topics/LockObject2d.md) | `FLOAT LockObject2d(HANDLE, HANDLE, FLOAT)` | 798 | Устанавливает у заданного двухмерного объекта флаг «Невыбираемый». |
| [LockSpace2d](../help/topics/LockSpace2d.md) | `FLOAT LockSpace2d(HANDLE, FLOAT)` | 245 | Функция позволяет заблокировать отрисовку окна при изменениях в графическом пространстве. |
| [ObjectToBottom2d](../help/topics/ObjectToBottom2d.md) | `FLOAT ObjectToBottom2d(HANDLE, HANDLE)` | 353 | Функция помещает двухмерный объект позади всех других (по Z-порядку). |
| [ObjectToTop2d](../help/topics/ObjectToTop2d.md) | `FLOAT ObjectToTop2d(HANDLE, HANDLE)` | 354 | Функция помещает двухмерный объект поверх всех других (по Z-порядку). |
| [OpenSchemeWindow](../help/topics/OpenSchemeWindow.md) | `HANDLE OpenSchemeWindow(STRING, STRING, STRING)` | 201 | Функция создает окно с заданным именем и содержащее схему указанного класса. |
| [PasteFromClipboard2d](../help/topics/PasteFromClipboard2d.md) | `HANDLE PasteFromClipboard2d(HANDLE, FLOAT, FLOAT, FLOAT)` | 377 | Функция вставляет двухмерный объект из буфера обмена в пространство. |
| [RotateObject2d](../help/topics/RotateObject2d.md) | `FLOAT RotateObject2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 382 | Функция поворачивает двухмерный объект на определенный угол. Угол вычисляется относительно текущего положения, для всех объектов кроме текста. Для текста вычисляется абсолютный угол. |
| [SaveRectArea2d](../help/topics/SaveRectArea2d.md) | `FLOAT SaveRectArea2d(HANDLE, STRING, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 244 | Функция позволяет записать прямоугольную область графического окна в файл. |
| [SetBitmapSrcRect2d](../help/topics/SetBitmapSrcRect2d.md) | `FLOAT SetBitmapSrcRect2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 357 | Функция устанавливает фрагмент битовой карты, который отображается на экране. |
| [SetBkBrush2d](../help/topics/SetBkBrush2d.md) | `FLOAT SetBkBrush2d(HANDLE, HANDLE)` | 738 | Функция устанавливает у двухмерного пространства фоновую кисть. |
| [SetBrushColor2d](../help/topics/SetBrushColor2d.md) | `FLOAT SetBrushColor2d(HANDLE, HANDLE, COLORREF)` | 406 | Функция позволяет установить цвет кисти. |
| [SetBrushColors2d](../help/topics/SetBrushColors2d.md) | `FLOAT SetBrushColors2d(HANDLE, HANDLE, HANDLE)` | 1124 | Функция для установки массива цветов для градиентной кисти |
| [SetBrushDib2d](../help/topics/SetBrushDib2d.md) | `FLOAT SetBrushDib2d(HANDLE, HANDLE, HANDLE)` | 408 | Функция устанавливает дескриптор битовой карты для кисти. |
| [SetBrushHatch2d](../help/topics/SetBrushHatch2d.md) | `FLOAT SetBrushHatch2d(HANDLE, HANDLE, FLOAT)` | 396 | Функция устанавливет стиль штриховки кисти. |
| [SetBrushObject2d](../help/topics/SetBrushObject2d.md) | `HANDLE SetBrushObject2d(HANDLE, HANDLE, HANDLE)` | 309 | Функция устанавливает кисть, которая будет использоваться при рисовании линии. |
| [SetBrushPoints2d](../help/topics/SetBrushPoints2d.md) | `FLOAT SetBrushPoints2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1123 | Функция для изменения точек градиентной кисти графического объекта. Точки определяются относительно положения объекта. Т.е. точка |
| [SetBrushRop2d](../help/topics/SetBrushROP2d.md) | `FLOAT SetBrushRop2d(HANDLE, HANDLE, FLOAT)` | 407 | Функция устанавливает логическую операцию при заливке кистью. |
| [SetBrushStyle2d](../help/topics/SetBrushStyle2d.md) | `FLOAT SetBrushStyle2d(HANDLE, HANDLE, FLOAT)` | 409 | Функция устанавливает стиль кисти. |
| [SetCrdSystem2d](../help/topics/SetCrdSystem2d.md) | `FLOAT SetCrdSystem2d(HANDLE, FLOAT, FLOAT)` | 89 | Функция устанавливает в пространстве новую систему координат. |
| [SetCurrentObject2d](../help/topics/SetCurrentObject2d.md) | `FLOAT SetCurrentObject2d(HANDLE, HANDLE)` | 666 | Функция позволяет установить текущий двухмерный объект. |
| [SetDDibObject2d](../help/topics/SetDDibObject2d.md) | `FLOAT SetDDibObject2d(HANDLE, HANDLE, HANDLE)` | 747 | Функция позволяет установить другую двойную битовую карту, используемую данным двухмерным объектом |
| [SetDibObject2d](../help/topics/SetDibObject2d.md) | `FLOAT SetDibObject2d(HANDLE, HANDLE, HANDLE)` | 745 | Функция позволяет установить другую битовую карту, используемую графическим объектом |
| [SetFontName2d](../help/topics/SetFontName2d.md) | `FLOAT SetFontName2d(HANDLE, HANDLE, STRING)` | 852 | Функция позволяет установить произвольное имя для заданного шрифта |
| [SetFontSize2d](../help/topics/SetFontSize2d.md) | `FLOAT SetFontSize2d(HANDLE, HANDLE, FLOAT)` | 850 | Функция позволяет установить размер шрифта в пунктах (логических единицах) |
| [SetFontStyle2d](../help/topics/SetFontStyle2d.md) | `FLOAT SetFontStyle2d(HANDLE, HANDLE, FLOAT)` | 851 | Функция позволяет установить стиль шрифта |
| [SetGroupItem2d](../help/topics/SetGroupItem2d.md) | `FLOAT SetGroupItem2d(HANDLE, HANDLE, FLOAT, HANDLE)` | 374 | Функция осуществляет замену указанного объекта в группе на другой. |
| [SetGroupItems2d](../help/topics/SetGroupItems2d.md) | `FLOAT SetGroupItems2d(HANDLE, HANDLE, [HANDLE])` | 375 | Функция меняет несколько объектов в группе на другие. Имеет переменное число аргументов для задания дескрипторов объектов. |
| [SetLineArrows2d](../help/topics/SetLineArrows2d.md) | `FLOAT SetLineArrows2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 299 | Функция позволяет установить концевую и начальную стрелку у двухмерной полилинии |
| [SetObjectAlpha2d](../help/topics/SetDibPixel2d.md) | `FLOAT SetObjectAlpha2d(HANDLE, HANDLE, FLOAT)` | 1121 | Функция для изменения прозрачности графического объекта |
| [SetObjectAttribute2d](../help/topics/SetObjectAttribute2d.md) | `FLOAT SetObjectAttribute2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 358 | Установить для графического объекта атрибуты. |
| [SetObjectName2d](../help/topics/SetObjectName2d.md) | `FLOAT SetObjectName2d(HANDLE, HANDLE, STRING)` | 379 | Функция устанавливает графическому объекту новое имя. |
| [SetObjectOrg2d](../help/topics/SetObjectOrg2d.md) | `FLOAT SetObjectOrg2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 331 | Функция перемещает объект в новые координаты. |
| [SetObjectSize2d](../help/topics/SetObjectSize2d.md) | `FLOAT SetObjectSize2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 332 | Функция изменяет размер объекта. |
| [SetPenColor2d](../help/topics/SetPenColor2d.md) | `FLOAT SetPenColor2d(HANDLE, HANDLE, COLORREF)` | 318 | Функция устанавливает цвет карандаша. |
| [SetPenObject2d](../help/topics/SetPenObject2d.md) | `HANDLE SetPenObject2d(HANDLE, HANDLE, HANDLE)` | 310 | Функция устанавливает карандаш, который будет использоваться при рисовании линии. |
| [SetPenROP2d](../help/topics/SetPenRop2d.md) | `FLOAT SetPenROP2d(HANDLE, HANDLE, FLOAT)` | 319 | Функция устанавливает логическую операцию при рисовании карандашом. |
| [SetPenStyle2d](../help/topics/SetPenStyle2d.md) | `FLOAT SetPenStyle2d(HANDLE, HANDLE, FLOAT)` | 320 | Функция устанавливает стиль карандаша. |
| [SetPenWidth2d](../help/topics/SetPenWidth2d.md) | `FLOAT SetPenWidth2d(HANDLE, HANDLE, FLOAT)` | 321 | Функция устанавливает толщину карандаша. |
| [SetRGNCreateMode](../help/topics/SetRGNCreateMode.md) | `FLOAT SetRGNCreateMode(HANDLE, HANDLE, FLOAT)` | 311 | Функция устанавливает режим заполнения региона. |
| [SetScaleSpace2d](../help/topics/SetScaleSpace2d.md) | `FLOAT SetScaleSpace2d(HANDLE, FLOAT)` | 344 | Функция устанавливает масштаб отображения двухмерного пространства. |
| [SetShowObject2d](../help/topics/SetShowObject2d.md) | `FLOAT SetShowObject2d(HANDLE, HANDLE, FLOAT)` | 364 | Функция позволяет спрятать или показать объект. |
| [SetSpaceLayers2d](../help/topics/SetSpaceLayers2d.md) | `FLOAT SetSpaceLayers2d(HANDLE, FLOAT)` | 487 | Функция устанавливает видимость слоев в двухмерном пространстве. |
| [SetSpaceOrg2d](../help/topics/SetSpaceOrg2d.md) | `FLOAT SetSpaceOrg2d(HANDLE, FLOAT, FLOAT)` | 343 | Функция сдвигает графическое пространство так, чтобы указанная точка соответствовала левому верхнему углу окна. |
| [SetSpaceRenderEngine2d](../help/topics/SetSpaceRenderEngine2d.md) | `FLOAT SetSpaceRenderEngine2d(HANDLE, FLOAT)` | 1130 | Функция устанавливает движок рендеринга графического пространства. Возможно использовать движок GDI (engine=0) или Cairo (engine=1), в котором добавлена возможность сглаживания векторных объектов, использование градиентной заливки, полупрозрачность графических объектов, вставка файлов png и т.д. |
| [SetString2d](../help/topics/SetString2d.md) | `FLOAT SetString2d(HANDLE, HANDLE, STRING)` | 365 | Функция изменяет строку текста |
| [SetText2d](../help/topics/SetText2d.md) | `FLOAT SetText2d(HANDLE, HANDLE, HANDLE, HANDLE, COLORREF, COLORREF)`<br>`FLOAT SetText2d(HANDLE, HANDLE, FLOAT, HANDLE, HANDLE, COLORREF, COLORREF)` | 602, 861 | Функция позволяет изменить одновременно шрифт, строку и цвет элемента логического текста |
| [SetTextBkColor2d](../help/topics/SetTextBkColor2d.md) | `FLOAT SetTextBkColor2d(HANDLE, HANDLE, FLOAT, COLORREF)` | 860 | Функция позволяет изменить цвет фона элемента логического текста |
| [SetTextFgColor2d](../help/topics/SetTextFgColor2d.md) | `FLOAT SetTextFgColor2d(HANDLE, HANDLE, FLOAT, COLORREF)` | 859 | Функция позволяет изменить цвет букв элемента логического текста |
| [SetTextFont2d](../help/topics/SetTextFont2d.md) | `FLOAT SetTextFont2d(HANDLE, HANDLE, FLOAT, HANDLE)` | 858 | Функция позволяет изменить шрифт элемента логического текста |
| [SetTextString2d](../help/topics/SetTextString2d.md) | `FLOAT SetTextString2d(HANDLE, HANDLE, FLOAT, HANDLE)` | 857 | Функция позволяет изменить строку элемента логического текста |
| [SetVectorPoint2d](../help/topics/SetVectorPoint2d.md) | `FLOAT SetVectorPoint2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 312 | Функция изменяет координаты одной из точек в линии. |
| [SetZOrder2d](../help/topics/SetZOrder2d.md) | `FLOAT SetZOrder2d(HANDLE, HANDLE, FLOAT)` | 355 | Функция перемещает двухмерный объект в заданное место Z списка. |
| [ShowObject2d](../help/topics/ShowObject2d.md) | `ShowObject2d(HANDLE, HANDLE)` | 383 | Функция устанавливает видимость графического объекта. Обратное действие производит функция [HideObject2d](HideObject2d.md). |
| [SwapObject2d](../help/topics/SwapObject2d.md) | `FLOAT SwapObject2d(HANDLE, HANDLE, HANDLE)` | 356 | Функция меняет два графических объекта местами в Z списке. |

## Без группы (только в таблицах компилятора) (126)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| AddControlText2d | `FLOAT AddControlText2d(HANDLE, HANDLE, STRING)`<br>`FLOAT AddControlText2d(HANDLE, HANDLE, STRING, FLOAT)` | 1105, 1106 |  |
| AnalyseWord | `STRING AnalyseWord(STRING)` | 895 |  |
| ApplyTexture3d | `FLOAT ApplyTexture3d(HANDLE, HANDLE, HANDLE, HANDLE, [FLOAT])` | 728 |  |
| BillboardSet_CreateBillboard | `HANDLE BillboardSet_CreateBillboard(HANDLE, FLOAT, FLOAT, FLOAT)` | 1079 |  |
| BillboardSet_GetBillboard | `HANDLE BillboardSet_GetBillboard(HANDLE, FLOAT)` | 1081 |  |
| BillboardSet_GetCommonDirection | `BillboardSet_GetCommonDirection(HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 1086 |  |
| BillboardSet_GetNumBillboards | `FLOAT BillboardSet_GetNumBillboards(HANDLE)` | 1080 |  |
| BillboardSet_RemoveBillboard | `BillboardSet_RemoveBillboard(HANDLE, HANDLE)` | 1082 |  |
| Change | `STRING Change(STRING, STRING, STRING)` | 119 |  |
| CheckMenuItem | `FLOAT CheckMenuItem(STRING, FLOAT, FLOAT)` | 762 |  |
| ChooseFolderDialog | `STRING ChooseFolderDialog(STRING, STRING, FLOAT)` | 99 |  |
| ChoseColorDialog | `COLORREF ChoseColorDialog(STRING, COLORREF)` | 227 |  |
| Compare | `FLOAT Compare(STRING, STRING)` | 134 |  |
| CopyUserResult | `FLOAT CopyUserResult()`<br>`FLOAT CopyUserResult(HANDLE)` | 880, 887 |  |
| Create3dView2d | `HANDLE Create3dView2d()` | 662 |  |
| CreateFont2dpt | `HANDLE CreateFont2dpt(HANDLE, STRING, FLOAT, FLOAT)` | 862 |  |
| CreateLine2d | `HANDLE CreateLine2d(HANDLE, HANDLE, HANDLE, FLOAT, FLOAT)` | 300 |  |
| DbGetFieldCount | `FLOAT DbGetFieldCount(HANDLE)` | 550 |  |
| DbGetPos | `FLOAT DbGetPos(HANDLE)` | 582 |  |
| DecryptStream | `FLOAT DecryptStream(HANDLE, HANDLE)` | 1128 |  |
| dequation | `dequation(FLOAT)` | 743 |  |
| Dialog | `FLOAT Dialog(STRING, STRING, STRING)` | 605 |  |
| [DialogBox](../help/topics/DialogBox.md) | `FLOAT DialogBox(STRING, HANDLE)` | 608 | Функция создает диалог пользователя (см. [диалоги пользователя](Dialogs.md)). |
| DialogEx | `FLOAT DialogEx(STRING, [STRING, STRING, STRING])` | 606 |  |
| diff0 | `diff0(&FLOAT, &FLOAT, &FLOAT, FLOAT)` |  |  |
| diff1 | `FLOAT diff1(FLOAT, FLOAT, FLOAT)` | 740 |  |
| diff2 | `FLOAT diff2(FLOAT, FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT, FLOAT)` | 742 |  |
| DLLFunction | `DLLFunction()` | 479 |  |
| DuplicateObject3d | `HANDLE DuplicateObject3d(HANDLE, HANDLE)` | 761 |  |
| EnableMenuItem | `FLOAT EnableMenuItem(STRING, FLOAT, FLOAT)` | 763 |  |
| EncryptStream | `FLOAT EncryptStream(HANDLE, HANDLE)` | 1126 |  |
| Entity_GetMaterial | `Entity_GetMaterial(HANDLE, FLOAT, &STRING)` | 1267 |  |
| equation | `equation(FLOAT)` | 741 |  |
| [Float](../help/topics/Operator_sub.md) | `FLOAT Float(STRING)`<br>`FLOAT float(INTEGER)`<br>`FLOAT FLOAT(HANDLE)`<br>`FLOAT FLOAT(COLORREF)` | 143, 17, 17, 17 | Оператор вычисляет разницу между двумя числами. |
| FrameGetPos2d | `FLOAT FrameGetPos2d(HANDLE, HANDLE)` | 455 |  |
| GetActualWidth2d | `FLOAT GetActualWidth2d(HANDLE, HANDLE)` | 242 |  |
| GetControlText2ds | `FLOAT GetControlText2ds(HANDLE, HANDLE, HANDLE)` | 713 |  |
| GetControlTextLength2d | `FLOAT GetControlTextLength2d(HANDLE, HANDLE)` | 1103 |  |
| GetElement | `FLOAT GetElement(FLOAT, [FLOAT])` | 480 |  |
| GethObject | `HANDLE GethObject()` | 223 |  |
| GETLASTMCIERROR | `FLOAT GETLASTMCIERROR()` | 253 |  |
| [GetObjectBase3d](../help/topics/GetObjectBase3d.md) | `FLOAT GetObjectBase3d(HANDLE, HANDLE, &FLOAT, &FLOAT, &FLOAT)` | 651 | Функция позволяет определить координаты начала координат объекта в текущей системе координат. |
| GetProjectProp | `FLOAT GetProjectProp(STRING, STRING)` | 726 |  |
| GetRDib2d | `STRING GetRDib2d(HANDLE, HANDLE)` | 758 |  |
| GetRDoubleDib2d | `STRING GetRDoubleDib2d(HANDLE, HANDLE)` | 759 |  |
| [GetSentanceTree](../help/topics/4O43S..md) | `STRING GetSentanceTree(STRING)` | 833 | Функция возвращает структуру предложения Sentence в виде дерева. Это дерево выводится в строку, где каждая вершина со смежными ей другими вершинами заключаются в квадратные скобки. |
| GetTextBkColor2d | `COLORREF GetTextBkColor2d(HANDLE, HANDLE)`<br>`COLORREF GetTextBkColor2d(HANDLE, HANDLE, FLOAT)` | 604, 856 |  |
| GetTextFgColor2d | `COLORREF GetTextFgColor2d(HANDLE, HANDLE)`<br>`COLORREF GetTextFgColor2d(HANDLE, HANDLE, FLOAT)` | 603, 855 |  |
| GetUserKeyFullValue | `STRING GetUserKeyFullValue(STRING)`<br>`STRING GetUserKeyFullValue(HANDLE, STRING)` | 877, 885 |  |
| GetUserKeyValue | `STRING GetUserKeyValue(STRING)`<br>`STRING GetUserKeyValue(HANDLE, STRING)` | 876, 884 |  |
| GetVarC | `COLORREF GetVarC(STRING, STRING)` | 432 |  |
| GetVarF | `FLOAT GetVarF(STRING, STRING)` | 430 |  |
| GetVarH | `HANDLE GetVarH(STRING, STRING)` | 432 |  |
| [GetVideoMarker](../help/topics/GetVideoMarker.md) | `FLOAT GetVideoMarker(HANDLE, COLORREF, COLORREF, COLORREF, &FLOAT, &FLOAT)` | 749 | Функция позволяет определить положение заданного цветового фрагмента в заданном видеокадре. |
| integer | `INTEGER integer(FLOAT)` | 16 |  |
| IsProjectExist | `FLOAT IsProjectExist(STRING)` | 723 |  |
| jmp | `jmp()` | 51 |  |
| jnz | `jnz(FLOAT)`<br>`jnz(HANDLE)` | 52, 110 |  |
| jz | `jz(FLOAT)`<br>`jz(HANDLE)` | 53, 111 |  |
| LBGetCount | `FLOAT LBGetCount(HANDLE, HANDLE)` | 467 |  |
| LBGetSelIndex | `FLOAT LBGetSelIndex(HANDLE, HANDLE)` | 468 |  |
| LBGetSelIndexs | `HANDLE LBGetSelIndexs(HANDLE, HANDLE)` | 1114 |  |
| LBInsertString | `FLOAT LBInsertString(HANDLE, HANDLE, STRING, FLOAT)` | 463 |  |
| LoadProject | `FLOAT LoadProject(STRING)` | 720 |  |
| Lower | `STRING Lower(STRING)` | 130 |  |
| Ltrim | `STRING Ltrim(STRING)` | 137 |  |
| MAddColumn | `FLOAT MAddColumn(FLOAT, FLOAT)` | 102 |  |
| MAddRow | `FLOAT MAddRow(FLOAT, FLOAT)` | 103 |  |
| MCISENDSTRINGEX | `STRING MCISENDSTRINGEX(STRING, FLOAT)` | 252 |  |
| MCISENDSTRINGstr | `STRING MCISENDSTRINGstr(STRING)` | 250 |  |
| MorphDivide | `STRING MorphDivide(STRING)` | 897 |  |
| NUI_CreateInstance | `HANDLE NUI_CreateInstance(FLOAT)` | 1503 |  |
| NUI_DestroyInstance | `NUI_DestroyInstance(HANDLE)` | 1504 |  |
| NUI_GetDeviceCount | `FLOAT NUI_GetDeviceCount()` | 1502 |  |
| NUI_GetDeviceName | `STRING NUI_GetDeviceName(HANDLE)` | 1505 |  |
| NUI_GetSkeletonPositions | `FLOAT NUI_GetSkeletonPositions(HANDLE, HANDLE, FLOAT)` | 1507 |  |
| NUI_Init | `NUI_Init()` | 1501 |  |
| NUI_InitInstance | `FLOAT NUI_InitInstance(HANDLE, FLOAT, FLOAT, FLOAT)` | 1506 |  |
| Overlay_FindElementAt | `HANDLE Overlay_FindElementAt(HANDLE, FLOAT, FLOAT)` | 1240 |  |
| ParticleSystem_CreateEx | `HANDLE ParticleSystem_CreateEx(HANDLE, STRING, STRING)` | 1218 |  |
| push | `FLOAT push()`<br>`STRING push()`<br>`INTEGER push()`<br>`HANDLE push()`<br>`COLORREF push()` | 1, 120, 3, 3, 3 |  |
| push_cst | `FLOAT push_cst()`<br>`STRING push_cst()`<br>`INTEGER push_cst()`<br>`HANDLE push_cst()` | 6, 122, 5, 5 |  |
| push_new | `FLOAT push_new()`<br>`STRING push_new()`<br>`INTEGER push_new()`<br>`HANDLE push_new()`<br>`COLORREF push_new()` | 2, 121, 4, 4, 4 |  |
| randomize | `randomize(FLOAT)` | 177 |  |
| ReadProjectKey | `FLOAT ReadProjectKey(HANDLE, STRING)` | 888 |  |
| ReadUserKey | `HANDLE ReadUserKey(STRING)` | 883 |  |
| RemoveText2d | `FLOAT RemoveText2d(HANDLE, HANDLE, FLOAT)` | 1112 |  |
| RemoveTexture3d | `FLOAT RemoveTexture3d(HANDLE, HANDLE)` | 729 |  |
| RenderWindow_Create | `HANDLE RenderWindow_Create(HANDLE, FLOAT, FLOAT)` | 1022 |  |
| RenderWindow_Create2 | `HANDLE RenderWindow_Create2(HANDLE, STRING, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 1304 |  |
| RenderWindow_CreateEx | `HANDLE RenderWindow_CreateEx(HANDLE, FLOAT, FLOAT, FLOAT)` | 1075 |  |
| [RenderWindow_Destroy](../help/topics/RenderWindow_Destroy.md) | `RenderWindow_Destroy(HANDLE)` | 1024 | Функция удаления окна рендеринга. |
| RenderWindow_GetCursorHovered | `FLOAT RenderWindow_GetCursorHovered(HANDLE)` | 1296 |  |
| Replicate | `STRING Replicate(STRING, FLOAT)` | 129 |  |
| RGB | `COLORREF RGB(FLOAT, FLOAT, FLOAT)` | 150 |  |
| RGBEx | `COLORREF RGBEx(FLOAT, FLOAT, FLOAT, FLOAT)` | 199 |  |
| Right | `STRING Right(STRING, FLOAT)` | 126 |  |
| roundt | `FLOAT roundt(FLOAT, FLOAT)` | 1129 |  |
| SceneNode_GetScene | `HANDLE SceneNode_GetScene(HANDLE)` | 1217 |  |
| ScreenShot | `HANDLE ScreenShot(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)`<br>`HANDLE ScreenShot(HANDLE, HANDLE)`<br>`HANDLE ScreenShot(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)`<br>`HANDLE ScreenShot(HANDLE)` | 797, 842, 844, 845 |  |
| SelectViewCrd3d | `FLOAT SelectViewCrd3d(HANDLE, HANDLE)` | 675 |  |
| SendData | `FLOAT SendData(STRING, FLOAT)` | 1127 |  |
| SendUserResult | `FLOAT SendUserResult(HANDLE)`<br>`FLOAT SendUserResult(HANDLE, HANDLE)` | 879, 886 |  |
| SetActiveProject | `FLOAT SetActiveProject(STRING)` | 722 |  |
| SetControlText2ds | `FLOAT SetControlText2ds(HANDLE, HANDLE, HANDLE)` | 714 |  |
| SetControlTextColor2d | `FLOAT SetControlTextColor2d(HANDLE, HANDLE, COLORREF)` | 1102 |  |
| SetDibPixel2d | `FLOAT SetDibPixel2d(HANDLE, HANDLE, FLOAT, FLOAT, COLORREF)` | 1125 |  |
| SetElement | `FLOAT SetElement(FLOAT, [FLOAT])` | 481 |  |
| SetHyperJump2d | `FLOAT SetHyperJump2d(HANDLE, HANDLE, FLOAT, [STRING])` | 755 |  |
| SetMorphDivide | `FLOAT SetMorphDivide(STRING, STRING)` | 898 |  |
| SetPoints2d | `FLOAT SetPoints2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 334 |  |
| SetProjectProp | `FLOAT SetProjectProp(STRING, STRING, FLOAT)`<br>`FLOAT SetProjectProp(STRING, STRING, STRING)` | 725, 727 |  |
| SetRDib2d | `FLOAT SetRDib2d(HANDLE, HANDLE, STRING)` | 756 |  |
| SetRDoubleDib2d | `FLOAT SetRDoubleDib2d(HANDLE, HANDLE, STRING)` | 757 |  |
| SetStringBufferMode | `SetStringBufferMode(FLOAT)` | 1115 |  |
| SetWindowParent | `FLOAT SetWindowParent(HANDLE, HANDLE)` | 1108 |  |
| systemstr | `STRING systemstr(FLOAT)` | 152 |  |
| TextureUnitState_GetTexture | `TextureUnitState_GetTexture(HANDLE, STRING, HANDLE)` | 1065 |  |
| UnloadProject | `FLOAT UnloadProject(STRING)` | 721 |  |
| Upper | `STRING Upper(STRING)` | 131 |  |
| UserKeyIsAutorized | `FLOAT UserKeyIsAutorized()`<br>`FLOAT UserKeyIsAutorized(HANDLE)` | 889, 890 |  |
| VFunction | `VFunction()` | 478 |  |
| [VideoGetPos2d](../help/topics/VideoGetPos2d.md) | `FLOAT VideoGetPos2d(HANDLE)` | 454 | Функция определяет текущую позицию видеопотока. |
| Viewport_SetDimensions | `Viewport_SetDimensions(HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 1071 |  |
| WindowInTaskBar | `FLOAT WindowInTaskBar(HANDLE, FLOAT)` | 791 |  |
| WordDivide | `STRING WordDivide(STRING, STRING)` | 899 |  |

## Функции работы с 3D-графикой (46)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [_CameraProc3d](../help/topics/_CameraProc3d.md) | `FLOAT _CameraProc3d(STRING, HANDLE, HANDLE, FLOAT)` | 499 |  |
| [AddPoint3d](../help/topics/AddPoint3d.md) | `FLOAT AddPoint3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 644 | Функция добавляет новую точку к первичному трехмерному объекту. |
| [AddPrimitive3d](../help/topics/AddPrimitive3d.md) | `FLOAT AddPrimitive3d(HANDLE, HANDLE, FLOAT, COLORREF, [FLOAT])` | 641 | Функция позволяет добавить примитив в произвольный трехмерный объект. |
| [CreateDefCamera3d](../help/topics/CreateDefCamera3d.md) | `HANDLE CreateDefCamera3d(HANDLE, FLOAT)` | 657 | Функция позволяет создать новую камеру в заданном трехмерном пространстве. |
| [CreateMaterial3d](../help/topics/CreateMaterial3d.md) | `HANDLE CreateMaterial3d(HANDLE, STRING, STRING, COLORREF, COLORREF, COLORREF, COLORREF, FLOAT, FLOAT, FLOAT)` | 670 | Функция создает новый материал для трехмерных объектов. Для удаления созданного материала следует пользоваться функцией DeleteTool2d. Например: rez:=DeleteTool2d(HSpace3d ,TEXTURE3D,hMaterial) или |
| [CreateObject3d](../help/topics/CreateObject3d.md) | `HANDLE CreateObject3d(HANDLE)` | 638 | Функция создает пустой трехмерный объект. После создания объекта необходимо самому добавлять трехмерные точки и примитивы. |
| [CreateObjectFromFile3d](../help/topics/CreateObjectFromFile3d.md) | `HANDLE CreateObjectFromFile3d(HANDLE, STRING)` | 197 | Функция создает в трехмерном пространстве объект из указанного файла. |
| [CreateSpace3d](../help/topics/CreateSpace3d.md) | `HANDLE CreateSpace3d(HANDLE)` | 660 | Функция позволяет создать пустое трехмерное пространство. |
| [CreateSurface3d](../help/topics/CreateSurface3d.md) | `HANDLE CreateSurface3d(HANDLE, HANDLE, FLOAT, FLOAT, COLORREF, FLOAT)` | 668 | Функция позволяет создать трехмерную поверхность. |
| [DeleteSpace3d](../help/topics/DeleteSpace3d.md) | `FLOAT DeleteSpace3d(HANDLE)` | 661 | Функция позволяет удалить трехмерное пространство. |
| [DelPoint3d](../help/topics/DelPoint3d.md) | `FLOAT DelPoint3d(HANDLE, HANDLE, FLOAT)` | 642 | Функция удаляет произвольную точку у первичного трехмерного объекта. |
| [DelPrimitive3d](../help/topics/DelPrimitive3d.md) | `FLOAT DelPrimitive3d(HANDLE, HANDLE, FLOAT)` | 639 | Функция позволяет удалить заданный примитив из произвольного трехмерного объекта. |
| [FitToCamera3d](../help/topics/FitToCamera3d.md) | `FLOAT FitToCamera3d(HANDLE, HANDLE, HANDLE, FLOAT)` | 669 | Функция позволяет подобрать параметры камеры так, чтобы вся сцена поместилась на экране. |
| [GetActiveCamera3d](../help/topics/GetActiveCamera3D.md) | `HANDLE GetActiveCamera3d()` | 659 | Функция позволяет определить камеру, используемую текущей проекцией. |
| [GetColors3d](../help/topics/GetColors3d.md) | `FLOAT GetColors3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 678 | Функция позволяет получить цвета  примитивов у произвольного трехмерного объекта. |
| [GetMaterialByName3d](../help/topics/GetMaterialByName3d.md) | `HANDLE GetMaterialByName3d(HANDLE, STRING)` | 679 |  |
| [GetNumPoints3d](../help/topics/GetNumPoints3d.md) | `FLOAT GetNumPoints3d(HANDLE, HANDLE)` | 643 | Функция возвращает количество точек в трехмерном объекте. |
| [GetNumPrimitives3d](../help/topics/GetNumPrimitives3d.md) | `FLOAT GetNumPrimitives3d(HANDLE, HANDLE)` | 640 | Функция позволяет получить число примитивов у произвольного трехмерного объекта. |
| [GetObject3dFromPoint2d](../help/topics/GetObject3dFromPoint2d.md) | `HANDLE GetObject3dFromPoint2d(HANDLE, HANDLE, FLOAT, FLOAT, &HANDLE, &FLOAT)` | 296 | Получение трехмерного объекта по двухмерным координатам. |
| [GetObjectBase3dM](../help/topics/GetObjectBase3dM.md) | `FLOAT GetObjectBase3dM(HANDLE, HANDLE, FLOAT)` | 649 | Функция позволяет определить координаты начала координат объекта в текущей системе координат. |
| [GetObjectColor3d](../help/topics/GetObjectColor3d.md) | `COLORREF GetObjectColor3d(HANDLE, HANDLE)` | 665 | Функция позволяет определить цвет трехмерного объекта. |
| [GetObjectDimension3d](../help/topics/GetObjectDimension3d.md) | `FLOAT GetObjectDimension3d(HANDLE, HANDLE, FLOAT)` | 653 | Функция позволяет получить размеры трехмерного объекта |
| [GetObjectMatrix3d](../help/topics/GetObjectMatrix3d.md) | `FLOAT GetObjectMatrix3d(HANDLE, HANDLE, FLOAT)` | 646 | Функция позволяет вернуть матрицу трансформации у трехмерного объекта. |
| [GetObjectPoints3d](../help/topics/GetObjectPoints3d.md) | `FLOAT GetObjectPoints3d(HANDLE, HANDLE, FLOAT)` | 663 | Функция позволяет получить все точки трехмерного объекта и записать их в указанную матрицу |
| [GetPoint3d](../help/topics/GetPoint3d.md) | `FLOAT GetPoint3d(HANDLE, HANDLE, FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 635 | Функция позволяет получить произвольную точку трехмерного объекта. |
| [GetSpace3d](../help/topics/GetSpace3d.md) | `HANDLE GetSpace3d()` | 652 | Функция позволяет определить трехмерное графическое пространство, используемое в заданной проекции. |
| [PopCrdSystem3d](../help/topics/PopCrdSystem3d.md) | `FLOAT PopCrdSystem3d(HANDLE)` | 672 | Функция позволяет восстановить предыдущую систему координат, которая была запомнена функцией PushCrdSystem3d. |
| [PushCrdSystem3d](../help/topics/PushCrdSystem3d.md) | `FLOAT PushCrdSystem3d(HANDLE)` | 671 | Функция позволяет запомнить во внутреннем стеке трехмерного пространства текущую систему координат. |
| [RemoveTexture](../help/topics/RemoveTexture3d.md) | `FLOAT RemoveTexture(HANDLE HSpace3d, HANDLE HObject3d)` |  | Функция убирает текстуру с заданного трехмерного объекта |
| [RotateObject3d](../help/topics/RotateObject3d.md) | `FLOAT RotateObject3d(HANDLE, HANDLE, FLOAT, FLOAT)` | 655 | Функция позволяет повернуть трехмерный объект относительно любой оси на произвольный угол. |
| [RotateObjectPoints3d](../help/topics/RotateObjectPoints3d.md) | `FLOAT RotateObjectPoints3d(HANDLE, HANDLE, FLOAT, FLOAT)` | 648 |  |
| [SelectLocalCrd3d](../help/topics/SelectLocalCrd3d.md) | `FLOAT SelectLocalCrd3d(HANDLE, HANDLE)` | 673 | Функция выбирает в качестве текущей локальную систему координат. |
| [SelectWorldCrd3d](../help/topics/SelectWorldCrd3d.md) | `FLOAT SelectWorldCrd3d(HANDLE)` | 674 | Функция выбирает в качестве текущей мировую систему координат. |
| [SetCameraPoint3d](../help/topics/SetCameraPoint3d.md) | `FLOAT SetCameraPoint3d(HANDLE hSpace3d, HANDLE hCamera3d, FLOAT x, FLOAT y, FLOAT z, FLOAT point)` |  |  |
| [SetColors3d](../help/topics/SetColors3d.md) | `FLOAT SetColors3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 677 | Функция позволяет установить цвета  примитивов у произвольного трехмерного объекта. |
| [SetObjectBase3d](../help/topics/SetObjectBase3d.md) | `FLOAT SetObjectBase3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT)` | 650 | Функция перемещает объект так, что точка начала его координат окажется в точке (x,y,z). |
| [SetObjectColor3d](../help/topics/SetObjectColor3d.md) | `FLOAT SetObjectColor3d(HANDLE, HANDLE, COLORREF)` | 656 | Функция позволяет изменить цвет трехмерного объекта. |
| [SetObjectMatrix3d](../help/topics/SetObjectMatrix3d.md) | `FLOAT SetObjectMatrix3d(HANDLE, HANDLE, FLOAT)` | 645 | Функция позволяет установить матрицу трансформации у трехмерного объекта. |
| [SetObjectPoints3d](../help/topics/SetObjectPoints3d.md) | `FLOAT SetObjectPoints3d(HANDLE, HANDLE, FLOAT)` | 664 | Функция позволяет установить все точки трехмерного объекта взяв из указанной матрицы. |
| [SetPoint3d](../help/topics/SetPoint3d.md) | `FLOAT SetPoint3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 636 | Функция позволяет изменить произвольную точку трехмерного объекта. |
| [SetPrimitive3d](../help/topics/SetPrimitive3d.md) | `FLOAT SetPrimitive3d(HANDLE, HANDLE, FLOAT, FLOAT, COLORREF, [FLOAT])` | 637 | Параметры |
| [SweepAndExtrude3d](../help/topics/SweepAndExtrude3d.md) | `HANDLE SweepAndExtrude3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, COLORREF, FLOAT)` | 297 | Функция создает трехмерную поверхность, на базе исходного трехмерного контура, вытягивая его в произвольном направлении и/или вращая вокруг заданной оси. |
| [SwitchToCamera3d](../help/topics/SwitchToCamera3d.md) | `FLOAT SwitchToCamera3d()` | 658 | Функция позволяет переключиться на новую камеру. |
| [TransformCamera3d](../help/topics/TransformCamera3d.md) | `FLOAT TransformCamera3d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 676 | Изменение параметров проекции |
| [TransformObject3d](../help/topics/TransformObject3d.md) | `FLOAT TransformObject3d(HANDLE, HANDLE, FLOAT)` | 654 | Функция позволяет произвольно изменить трехмерный объект. |
| [TransformObjectPoints3d](../help/topics/TransformObjectPoints3d.md) | `FLOAT TransformObjectPoints3d(HANDLE, HANDLE, FLOAT)` | 647 |  |

## Функции работы с базами данных (45)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [DbAddIndex](../help/topics/DbAddIndex.md) | `FLOAT DbAddIndex(HANDLE, STRING, FLOAT, STRING, FLOAT, FLOAT, STRING, STRING, [STRING])` | 530 | Функция DbAddIndex создает индекс у существующей таблицы или подключает существующий индекс к таблице. |
| [DbAppendRecord](../help/topics/DbAppendRecord.md) | `FLOAT DbAppendRecord(HANDLE)` | 520 | Функция добавляет запись в конец таблицы. Все значения новой записи устанавливаются по умолчанию. |
| [DbCloseAll](../help/topics/DbCloseAll.md) | `DbCloseAll()` | 507 | Функция закрывает все открытые базы и таблицы. |
| [DbCloseBase](../help/topics/DbCloseBase.md) | `FLOAT DbCloseBase(HANDLE)` | 502 | Функция закрывает указанную базу данных и все открытые в ней таблицы. |
| [DbCloseIndex](../help/topics/DbCloseIndex.md) | `FLOAT DbCloseIndex(HANDLE, STRING)` | 534 | Функция DbCloseIndex позволяет закрыть индекс. |
| [DbCloseTable](../help/topics/DbCloseTable.md) | `FLOAT DbCloseTable(HANDLE)` | 503 | Функция закрывает указанную таблицу или курсор |
| [DbCopyTo](../help/topics/DbCopyTo.md) | `FLOAT DbCopyTo(HANDLE, STRING, STRING)` | 548 | Функция позволяет скопировать заданную таблицу в другую таблицу произвольного типа. |
| [DbCreateTable](../help/topics/DbCreateTable.md) | `FLOAT DbCreateTable(HANDLE, STRING, STRING, STRING, FLOAT)` | 528 | Функция пока не работает. |
| [DbDeleteIndex](../help/topics/DbDeleteIndex.md) | `FLOAT DbDeleteIndex(HANDLE, STRING, FLOAT, STRING)` | 531 | Функция DbDeleteIndex физически удаляет индекс с диска. |
| [DbDeleteRecord](../help/topics/DbDeleteRecord.md) | `FLOAT DbDeleteRecord(HANDLE)` | 521 | Функция удаляет запись из таблицы текущую запись |
| [DbFieldId](../help/topics/DbFieldId.md) | `FLOAT DbFieldId(HANDLE, STRING)` | 511 | Функция DbFieldId позволяет получить индекс поля по его имени. |
| [DbFreeBlob](../help/topics/DbFreeBlob.md) | `FLOAT DbFreeBlob(HANDLE, FLOAT)` | 538 | Функция DBFreeBlob удаляет содержимое МЕМО поля. |
| [DbGetBlob](../help/topics/DbGetBlob.md) | `FLOAT DbGetBlob(HANDLE, FLOAT, HANDLE)` | 536 | Функция DBGetBlob позволяет переписать содержимое МЕМО поля в поток. |
| [DbGetCodePage](../help/topics/DbGetCodePage.md) | `FLOAT DbGetCodePage(HANDLE)` | 544 | Пока не реализована |
| [DbGetCount](../help/topics/DbGetCount.md) | `FLOAT DbGetCount(HANDLE)` | 514 | Функция возвращает количество записей в таблице. (реализовано только для DBase) |
| [DbGetDelMode](../help/topics/DbGetDelMode.md) | `FLOAT DbGetDelMode(HANDLE)` | 541 | Реализовано только для DBase |
| [DbGetError](../help/topics/DbGetError.md) | `FLOAT DbGetError()` | 504 | Функция возвращает код последней ошибки при работе с базами данных. |
| [DbGetErrorStr](../help/topics/DbGetErrorStr.md) | `STRING DbGetErrorStr(FLOAT)` | 505 | Функция возвращает строковое описание последней ошибки при работе с базами данных. |
| [DbGetField](../help/topics/DbGetField.md) | `STRING DbGetField(HANDLE, FLOAT)`<br>`STRING DbGetField(HANDLE, STRING)` | 512, 527 | Функция позволяет вернуть значение ячейки в строковом виде. |
| [DBGetFieldN](../help/topics/DbGetFieldN.md) | `FLOAT DBGetFieldN(HANDLE, FLOAT)`<br>`FLOAT DbGetFieldN(HANDLE, STRING)` | 516, 526 | Функция возвращает значение ячейки в числовом виде. |
| [DBGetFieldName](../help/topics/DbGetFieldName.md) | `STRING DBGetFieldName(HANDLE, FLOAT)` | 517 | Функция возвращает имя поля в таблице. |
| [DBGetFieldType](../help/topics/DbGetFieldType.md) | `FLOAT DBGetFieldType(HANDLE, FLOAT)` | 518 | Функция возвращает тип поля в таблице. |
| [DbGoBottom](../help/topics/DbGoBottom.md) | `FLOAT DbGoBottom(HANDLE)` | 509 | Функция устанавливает текущую позицию в конец таблицы. |
| [DbGoTop](../help/topics/DbGoTop.md) | `FLOAT DbGoTop(HANDLE)` | 508 | Функция устанавливает текущую позицию на начало. |
| [DbInsertRecord](../help/topics/DbInsertRecord.md) | `FLOAT DbInsertRecord(HANDLE)` | 519 | Функция добавляет запись в текущую позицию. Все значения новой записи устанавливаются по умолчанию. |
| [DbLock](../help/topics/DbLock.md) | `FLOAT DbLock(HANDLE, FLOAT)` | 545 | Функция позволяет установить текущий режим работы с таблицей. |
| [DbOpenBase](../help/topics/DbOpenBase.md) | `HANDLE DbOpenBase(STRING, STRING, STRING, STRING)` | 500 | Функция открывает базу данных. Должна быть вызвана перед тем, как будет открыта любая таблица в этой базе. |
| [DbOpenIndex](../help/topics/DbOpenIndex.md) | `FLOAT DbOpenIndex(HANDLE, STRING, FLOAT)` | 532 | Функция DbOpenIndex позволяет открыть уже существующий индекс |
| [DbOpenTable](../help/topics/DbOpenTable.md) | `HANDLE DbOpenTable(HANDLE, STRING, STRING, STRING, STRING, FLOAT, STRING)` | 501 | Функция открывает существующую таблицу и возвращает ее дескриптор. |
| [DbPackFile](../help/topics/DbPackFile.md) | `FLOAT DbPackFile(HANDLE)` | 539 | Функция DbPackFile пакует таблицу, физически удаляя записи, помеченные на удаление (реализовано только для DBase). |
| [DbPutBlob](../help/topics/DbPutBlob.md) | `FLOAT DbPutBlob(HANDLE, FLOAT, HANDLE)` | 537 | Функция DBPutBlob позволяет переписать содержимое потока в MEMO поле. |
| [DbRegenIndex](../help/topics/DbRegenIndex.md) | `FLOAT DbRegenIndex(HANDLE, STRING, FLOAT, STRING)` | 535 | Функция DbRegenIndex позволяет переиндексировать заданный индекс. |
| [DbSetCodePage](../help/topics/DbSetCodePage.md) | `FLOAT DbSetCodePage(HANDLE, FLOAT)` | 543 | пока не реализована |
| [DbSetDelMode](../help/topics/DbSetDelMode.md) | `FLOAT DbSetDelMode(HANDLE, FLOAT)` | 542 | Функция устанавливает режим удаления записей в таблице. Когда режим установлен в 1, удаленные записи можно восстановить функцией [DbUndeleteRecord](DbUndeleteRecord.md). |
| [DbSetDir](../help/topics/DbSetDir.md) | `FLOAT DbSetDir(HANDLE, STRING)` | 506 | Функция устанавливает директорий, в котором будут искаться таблицы базы данных. По умолчанию таким директорием считается директорий проекта. |
| [DbSetField](../help/topics/DbSetField.md) | `FLOAT DbSetField(HANDLE, FLOAT, STRING)`<br>`FLOAT DbSetField(HANDLE, FLOAT, FLOAT)`<br>`FLOAT DbSetField(HANDLE, STRING, STRING)`<br>`FLOAT DbSetField(HANDLE, STRING, FLOAT)` | 522, 523, 524, 525 | Функция изменяет значение поля в таблице. |
| [DbSetTable](../help/topics/DbSetTable.md) | `FLOAT DbSetTable(STRING, HANDLE, STRING)` | 580 | Функция сортирует таблицу по любым полям в заданном порядке. |
| [DbSetToKey](../help/topics/DbSetToKey.md) | `FLOAT DbSetToKey(HANDLE, STRING)` | 549 | Функция позволяет найти запись в проиндексированной таблице.( Временно |
| [DbSkip](../help/topics/DbSkip.md) | `FLOAT DbSkip(HANDLE, FLOAT)` | 510 | Функция перемещает текущую позицию на одну, к концу или началу таблицы, в зависимости от переменной Direction. Если перемещение невозможно из за выхода за пределы таблицы, функция возвращает нулевое значение. |
| [DbSortTable](../help/topics/DbSortTable.md) | `FLOAT DbSortTable(HANDLE, FLOAT, [STRING])` | 540 | Функция DbSortTable сортирует таблицу по любым полям в заданном порядке (не работает). |
| [DbSQL](../help/topics/DbSQL.md) | `HANDLE DbSQL(HANDLE, STRING, FLOAT)`<br>`HANDLE DbSQL(HANDLE, HANDLE, FLOAT)` | 513, 1120 | Функция выполняет SQL запрос (см. [Работа с SQL](SQL.md)). |
| [DbSwitchToIndex](../help/topics/DbSwitchToIndex.md) | `FLOAT DbSwitchToIndex(HANDLE, STRING, FLOAT, STRING, FLOAT)` | 533 | Функция DbSwitchToIndex позволяет сменить активный индекс. |
| [DbUndeleteRecord](../help/topics/DbUndeleteRecord.md) | `FLOAT DbUndeleteRecord(HANDLE)` | 547 | Функция позволяет отменить удаление текущей записи. (реализовано только для DBase) |
| [DbUnlock](../help/topics/DBUnlock.md) | `FLOAT DbUnlock(HANDLE, FLOAT)` | 546 | Разблокировать таблицу (см. [DbLock](DbLock.md)). |
| [DbZap](../help/topics/DbZap.md) | `FLOAT DbZap(HANDLE)` | 529 | Функция удаляет все записи в указанной таблице. |

## Системные функции (37)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [DeleteMenu](../help/topics/DeleteMenu.md) | `FLOAT DeleteMenu(STRING)` | 247 | Функция удаляет ранее установленное меню. |
| [GetAsyncKeyState](../help/topics/GetAsyncKeyState.md) | `FLOAT GetAsyncKeyState(FLOAT)` | 232 | Функция определяет - нажата ли указанная [виртуальная клавиша](Virtual_Keys.md). |
| [GetDate](../help/topics/GetDate.md) | `GetDate(&FLOAT, &FLOAT, &FLOAT)` | 748 | Функция позволяет определить текущую системную дату. |
| [GetFixedFrameHeight](../help/topics/GetFixedFrameHeight.md) | `FLOAT GetFixedFrameHeight()` | 788 | Позволяет определить толщину вертикальной рамки окна, размер которого нельзя менять, т.е. имеющего стиль WS_NORESIZE. |
| [GetFixedFrameWidth](../help/topics/GetFixedFrameWidth.md) | `FLOAT GetFixedFrameWidth()` | 787 | Позволяет определить толщину горизонтальной рамки окна, размер которого нельзя менять, т.е. имеющего стиль WS_NORESIZE. |
| [GetKeyboardLayout](../help/topics/GetKeyboardLayout.md) | `FLOAT GetKeyboardLayout()` | 778 | Позволяет определить текущую раскладку клавиатуры. |
| [GetMousePos](../help/topics/GetMousePos.md) | `FLOAT GetMousePos(STRING, &FLOAT, &FLOAT)` | 158 | Функция позволяет получить координаты курсора мышки относительно системы координат заданного окна. |
| [GetScreenHeight](../help/topics/GetScreenHeight.md) | `FLOAT GetScreenHeight()` | 773 | Позволяет определить высоту экрана, установленную в системном окне «Свойства: Экран» на вкладке «Параметры» в разделе «Разрешение экрана» |
| [GetScreenWidth](../help/topics/GetScreenWidth.md) | `FLOAT GetScreenWidth()` | 772 | Позволяет определить ширину экрана, установленную в системном окне «Свойства: Экран» на вкладке «Параметры» в разделе «Разрешение экрана» |
| [GetSizeFrameHeight](../help/topics/GetSizeFrameHeight.md) | `FLOAT GetSizeFrameHeight()` | 790 | Позволяет определить толщину вертикальной рамки окна, размер которого можно менять. |
| [GetSizeFrameWidth](../help/topics/GetSizeFrameWidth.md) | `FLOAT GetSizeFrameWidth()` | 789 | Позволяет определить толщину горизонтальной рамки окна, размер которого можно менять. |
| [GetSmallTitleHeight](../help/topics/GetSmallTitleHeight.md) | `FLOAT GetSmallTitleHeight()` | 786 | Позволяет определить высоту области уменьшенного заголовка  окон, например, как у панелей инструментов, открепленных от края окна. Такое окно можно создать, установив у него стиль WS_DIALOG \| WS_TOOL. |
| [gettickcount](../help/topics/GetTickCount.md) | `FLOAT gettickcount()` | 115 | Функция возвращает время в миллисекундах, прошедшее с момента запуска операционной системы. |
| [GetTime](../help/topics/GetTime.md) | `GetTime(&FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 475 | Функция позволяет определить текущее системное время. |
| [GetTitleHeight](../help/topics/GetTitleHeight.md) | `FLOAT GetTitleHeight()` | 785 | Позволяет определить высоту области заголовка обычных окон (перекрывающихся, диалоговых…) |
| [GetWorkAreaHeight](../help/topics/GetWorkAreaHeight.md) | `FLOAT GetWorkAreaHeight()` | 777 | Позволяет определить высоту рабочей области экрана без панели задач и других панелей, прикрепленных к краям рабочего стола. |
| [GetWorkAreaWidth](../help/topics/GetWorkAreaWidth.md) | `FLOAT GetWorkAreaWidth()` | 776 | Позволяет определить ширину рабочей области экрана без панели задач и других панелей, прикрепленных к краям рабочего стола. |
| [GetWorkAreaX](../help/topics/GetWorkAreaX.md) | `FLOAT GetWorkAreaX()` | 774 | Позволяет определить X-координату верхнего левого угла рабочей области экрана без панели задач и других панелей, прикрепленных к краям рабочего стола. |
| [GetWorkAreaY](../help/topics/GetWorkAreaY.md) | `FLOAT GetWorkAreaY()` | 775 | Позволяет определить Y-координату верхнего левого угла рабочей области экрана без панели задач и других панелей, прикрепленных к краям рабочего стола. |
| [joygetbuttons](../help/topics/JoyGetButtons.md) | `FLOAT joygetbuttons(FLOAT)` | 231 | Функция возвращает состояние кнопок джойстика. |
| [joygetx](../help/topics/JoyGetX.md) | `FLOAT joygetx(FLOAT)` | 228 | Функция возвращает одну из координат джойстика. |
| [joygety](../help/topics/JoyGetY.md) | `FLOAT joygety(FLOAT)` | 229 | Функция возвращает одну из координат джойстика. |
| [joygetz](../help/topics/JoyGetZ.md) | `FLOAT joygetz(FLOAT)` | 230 | Функция возвращает одну из координат джойстика. |
| [LoadCursor](../help/topics/LoadCursor.md) | `LoadCursor(HANDLE, STRING)`<br>`LoadCursor(STRING, STRING)` | 865, 866 | Функция позволяет установить для заданного окна курсор мыши, содержащийся в файле |
| [LoadMenu](../help/topics/LoadMenu.md) | `FLOAT LoadMenu(STRING, STRING, FLOAT)` | 246 | Устанавливается меню пользователя заданное специальным файлом - шаблоном. |
| [LogMessage](../help/topics/LogMessage.md) | `LogMessage(STRING)` | 491 | Функция выводит сообщение в окно сообщений |
| [SendMail](../help/topics/SendMail.md) | `SendMail(STRING)`<br>`FLOAT SendMail(STRING, FLOAT)` | 843, 1118 | Позволяет отправить электронное письмо через Интернет. Текст письма должен быть записан в текстовый файл Filename. Файл должен содержать теги: |
| [SendSMS](../help/topics/SendSMS.md) | `SendSMS(STRING, STRING)`<br>`FLOAT SendSMS(STRING, STRING, FLOAT)` | 841, 1117 | Позволяет отправить SMS-сообщение с помощью подключенного к компьютеру мобильного телефона. |
| [SetHyperJump](../help/topics/SetHyperJump2d.md) | `FLOAT SetHyperJump(HANDLE HSpace, HANDLE HObject, FLOAT mode [, STRING target, STRING window, STRING object, STRING effect])` |  | Функция позволяет изменить параметры гиперссылки. См. [Гипербаза](Hyperbase.md). |
| [SetStandartCursor](../help/topics/SetStandartCursor.md) | `SetStandartCursor(HANDLE, FLOAT)`<br>`SetStandartCursor(STRING, FLOAT)` | 863, 864 | Функция позволяет изменить курсор мыши для заданного окна. |
| [SetStatusText](../help/topics/SetStatusText.md) | `SetStatusText(FLOAT, STRING)` | 112 | Функция устанавливает текст для заданной ячейки строки статуса |
| [Shell](../help/topics/Shell.md) | `FLOAT Shell(STRING, STRING, STRING, FLOAT)` | 198 | Открывает произвольный файл или папку. |
| [ShellWait](../help/topics/ShellWait.md) | `FLOAT ShellWait(STRING, STRING, STRING, FLOAT)` | 882 | Открывает произвольный файл или папку и ждет окончания работы с этим файлом. Т.е. функция не вернет управление вызвавшей ее программе, пока работа с файлом или папкой не будет завершена. |
| [ShowCursor](../help/topics/ShowCursor.md) | `ShowCursor(FLOAT)` | 796 | Функция позволяет изменить видимость курсора мыши на экране. После остановки процесса моделирования видимость курсора восстанавливается автоматически |
| [StdHyperJump](../help/topics/StdHyperJump.md) | `StdHyperJump(HANDLE, FLOAT, FLOAT, HANDLE, FLOAT)` | 385 | Функция позволяет сымитировать нажатие мышкой на графический объект, являющийся элементом гипербазы. См. [гипербаза](Hyperbase.md). |
| [system](../help/topics/System.md) | `FLOAT system(FLOAT, [FLOAT])` | 151 | Получить различную информацию о состоянии системы. |
| [WinExecute](../help/topics/WinExecute.md) | `FLOAT WinExecute(STRING, FLOAT)` | 240 | Запускает указанную Windows программу. Для запуска приложения с аргументами необходимо использовать функцию [Shell](Shell.md). |

## Функции работы с мультимедиа (35)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [AudioGetBalance](../help/topics/AudioGetBalance.md) | `FLOAT AudioGetBalance(HANDLE)` | 812 | Функция определяет баланс каналов аудио-файла. |
| [AudioGetLength](../help/topics/AudioGetLength.md) | `FLOAT AudioGetLength(HANDLE)` | 818 | Функция определяет длину аудио-файла. |
| [AudioGetPosition](../help/topics/AudioGetPosition.md) | `FLOAT AudioGetPosition(HANDLE)` | 817 | Функция определяет текущую позицию аудио-файла. |
| [AudioGetRepeat](../help/topics/AudioGetRepeat.md) | `FLOAT AudioGetRepeat(HANDLE)` | 808 | Функция определяет режим повтора аудио-файла. |
| [AudioGetTone](../help/topics/AudioGetTone.md) | `FLOAT AudioGetTone(HANDLE)` | 814 | Функция определяет тональность аудио-файла. |
| [AudioGetVolume](../help/topics/AudioGetVolume.md) | `FLOAT AudioGetVolume(HANDLE)` | 810 | Функция определяет уровень громкости аудио-файла. |
| [AudioIsPlaying](../help/topics/AudioIsPlaying.md) | `FLOAT AudioIsPlaying(HANDLE)` | 805 | Функция проверяет воспроизводится ли в данный момент аудио-файл. |
| [AudioIsSeekable](../help/topics/AudioIsSeekable.md) | `FLOAT AudioIsSeekable(HANDLE)` | 815 | Функция проверяет можно ли позиционировать (прокручивать) аудио-файл. |
| [AudioOpenSound](../help/topics/AudioOpenSound.md) | `HANDLE AudioOpenSound(STRING)` | 802 | Функция открывает аудио-файл для дальнейшего воспроизведения. |
| [AudioPlay](../help/topics/AudioPlay.md) | `AudioPlay(HANDLE)` | 803 | Функция воспроизводит открытый аудио-файл. |
| [AudioReset](../help/topics/AudioReset.md) | `AudioReset(HANDLE)` | 806 | Функция возвращает аудио-файл в начало. |
| [AudioSetBalance](../help/topics/AudioSetBalance.md) | `AudioSetBalance(HANDLE, FLOAT)` | 811 | Функция устанавливает баланс каналов открытого аудио-файла, если это стерео-, а не моно-файл. |
| [AudioSetPosition](../help/topics/AudioSetPosition.md) | `AudioSetPosition(HANDLE, FLOAT)` | 816 | Функция устанавливает позицию открытого аудио-файла, т.е. выполняет его прокрутку, если аудио-файл это поддерживает. |
| [AudioSetRepeat](../help/topics/AudioSetRepeat.md) | `AudioSetRepeat(HANDLE, FLOAT)` | 807 | Функция устанавливает режим повтора аудио-файла. |
| [AudioSetTone](../help/topics/AudioSetTone.md) | `AudioSetTone(HANDLE, FLOAT)` | 813 | Функция устанавливает тональность открытого аудио-файла. |
| [AudioSetVolume](../help/topics/AudioSetVolume.md) | `AudioSetVolume(HANDLE, FLOAT)` | 809 | Функция устанавливает громкость воспроизведения аудио-файла. |
| [AudioStop](../help/topics/AudioStop.md) | `AudioStop(HANDLE)` | 804 | Функция останавливает воспроизведение аудио-файла. |
| [BeginWriteVideo2d](../help/topics/BeginWriteVideo2d.md) | `HANDLE BeginWriteVideo2d(HANDLE, STRING, FLOAT, FLOAT, FLOAT, FLOAT, STRING)` | 457 | Функция начинает запись видеофайла. Каждый кадр представляет собой прямоугольный фрагмент графического окна. |
| [CloseVideo](../help/topics/CloseVideo.md) | `FLOAT CloseVideo(HANDLE)` | 442 | Функция закрывает открытый видеопоток. Поток может быть закрыт только в том случае, если он не используется ни одним из видеофреймов. |
| [CreateVideoFrame2d](../help/topics/CreateVideoFrame2d.md) | `HANDLE CreateVideoFrame2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 446 | Функция создает графический объект - видеофрейм. |
| [FrameGetVideo2d](../help/topics/FrameGetVideo2d.md) | `HANDLE FrameGetVideo2d(HANDLE, HANDLE)` | 456 | Функция позволяет определить видеопоток подключенный к видеофрейму. |
| [FrameSetPos2d](../help/topics/FrameSetPos2d.md) | `FLOAT FrameSetPos2d(HANDLE, HANDLE, FLOAT)` | 448 | Функция устанавливает позицию у видеофрейма. |
| [FrameSetSrcRect2d](../help/topics/FrameSetSrcRect2d.md) | `FLOAT FrameSetSrcRect2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 453 | Функция позволяет установить прямоугольный фрагмент, отображаемый из видеокадра. |
| [GetLastMCIEerror](../help/topics/GetLastMCIError.md) | `FLOAT GetLastMCIEerror()` |  | функция возвращает код завершения последней функции MCISENDSTRING. |
| [GETMCIERRORSTR](../help/topics/GetMCIerrorStr.md) | `STRING GETMCIERRORSTR(FLOAT)` | 254 | Функция позволяет получить текстовое описание MCI ошибки, код которой вернули функции [GETLASTMCIERROR](GetLastMCIError.md), или [MCISENDSTRING](MCISendString.md). |
| [MCISENDSTRING](../help/topics/MCISendString.md) | `FLOAT MCISENDSTRING(STRING)` | 251 | Функция позволяет послать устройству управляющую строку (cм. [Интерфейс управляющих строк MCI](MCI_interface.md)). |
| [OpenVideo](../help/topics/OpenVideo.md) | `HANDLE OpenVideo(STRING, FLOAT)` | 441 | Функция открывает видеопоток. |
| [SndPlaySound](../help/topics/SndPlaySound.md) | `FLOAT SndPlaySound(STRING, FLOAT)` | 346 | Функция проигрывает Wav - файл. |
| [VideoDialog](../help/topics/VideoDialog.md) | `FLOAT VideoDialog(HANDLE)` | 458 | Функция вызывает диалог с настройками видеопотока. |
| [VideoPause2d](../help/topics/VideoPause2d.md) | `FLOAT VideoPause2d(HANDLE, HANDLE)` | 450 | Функция приостанавливает проигрывание видео. Для дальнейшего воспроизведения используется функция [VideoResume2d](VideoResume2d.md). |
| [VideoPlay2d](../help/topics/VideoPlay2d.md) | `FLOAT VideoPlay2d(HANDLE, HANDLE, FLOAT, FLOAT, FLOAT, FLOAT)` | 449 | Функция осуществляет проигрывание видео. |
| [VideoResume2d](../help/topics/VideoResume2d.md) | `FLOAT VideoResume2d(HANDLE, HANDLE)` | 451 | Функция возобновляет проигрывание видео, остановленного функцией [VideoPause2d](VideoPause2d.md). |
| [VideoSetPos2d](../help/topics/VideoSetPos2d.md) | `FLOAT VideoSetPos2d(HANDLE, FLOAT)` | 447 | Функция устанавливает позицию видеопотока. |
| [VideoStop2d](../help/topics/VideoStop2d.md) | `FLOAT VideoStop2d(HANDLE, HANDLE)` | 452 | Функция останавливает проигрывание видео. Для повторного запуска, необходимо использовать функцию [VideoPlay2d](VideoPlay2d.md). |
| [WriteVideoFrame2d](../help/topics/WriteVideoFrame2d.md) | `FLOAT WriteVideoFrame2d(HANDLE)` | 459 | Функция записывает один кадр в видеопоток (avi файл). |

## Математические функции (34)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [abs](../help/topics/Abs.md) | `FLOAT abs(FLOAT)` | 46 | Функция вычисляет абсолютное значение (модуль) аргумента. |
| [and](../help/topics/And.md) | `FLOAT and(FLOAT, FLOAT)` | 43 | Функция выполняет логическую операцию “и” над своими аргументами. |
| [arccos](../help/topics/Arccos.md) | `FLOAT arccos(FLOAT)` | 28 | Функция вычисляет значение угла, равного арккосинусу от аргумента. Эта функция является обратной к функции Cos, то есть Arccos(Cos(x)) равно x. В качестве параметра ей желательно передавать только числа от минус единицы до единицы, так как это предел изменения косинуса. В противном случае Arccos будет возвращать 0 для всех параметров больше единицы и pi для параметров меньше минус единицы. |
| [arcsin](../help/topics/Arcsin.md) | `FLOAT arcsin(FLOAT)` | 26 | Функция вычисляет значение угла, равного арксинусу от аргумента. Эта функция является обратной к функции sin, то есть Arcsin(sin(x)) равно x. В качестве параметра ей желательно передавать только числа от минус единицы до единицы, так как это предел изменения синуса. В противном случае Arcsin будет возвращать pi / 2 для всех параметров больше единицы и - pi / 2 для параметров меньше минус единицы. |
| [arctan](../help/topics/Arctan.md) | `FLOAT arctan(FLOAT)` | 24 | Функция вычисляет значение угла, равного арктангенсу от переданного ей аргумента. Эта функция является обратной к функции tg, то есть Arctan(tg(x)) равно x. |
| [average](../help/topics/Average.md) | `FLOAT average(FLOAT, FLOAT)` | 40 | Функция вычисляет среднее арифметическое двух переданных ей чисел. |
| [cos](../help/topics/Cos.md) | `FLOAT cos(FLOAT)` | 27 | Функция cos вычисляет значение косинуса угла, значение которого передано ей в качестве параметра. Угол задается в радианах. |
| [dec](../help/topics/Dec.md) | `dec(&FLOAT)`<br>`dec(&FLOAT, FLOAT)` | 869, 870 | Функция уменьшает значение аргумента value на величину step. Если аргумент step не указан (первый вариант функции), то value уменьшается на 1 |
| [deg](../help/topics/deg.md) | `FLOAT deg(FLOAT)` | 49 | Функция преобразует значение угла из радиан в градусы (180*arg/PI). |
| [delta](../help/topics/Delta.md) | `FLOAT delta(FLOAT)` | 37 | Функция Delta вычисляет дельта - функцию аргумента. |
| [ed](../help/topics/Ed.md) | `FLOAT ed(FLOAT)`<br>`FLOAT ed(HANDLE)` | 36, 84 | Функция вычисляет единичную функцию от переданного аргумента. |
| [exp](../help/topics/Exp.md) | `FLOAT exp(FLOAT)` | 33 | Функция Exp вычисляет экспоненту от переданного ей аргумента. |
| [GetAngleByXY](../help/topics/GetAngleByXY.md) | `FLOAT GetAngleByXY(FLOAT, FLOAT)` | 439 | Функция позволяет определить угол по значению координат X и Y c учетом октантов и т.д. |
| [inc](../help/topics/Inc.md) | `inc(&FLOAT)`<br>`inc(&FLOAT, FLOAT)` | 867, 868 | Функция увеличивает значение аргумента value на величину step. Если аргумент step не указан (первый вариант функции), то value увеличивается на 1 |
| [lg](../help/topics/Lg.md) | `FLOAT lg(FLOAT)` | 30 | Функция вычисляет десятичный логарифм от переданного ей аргумента. В качестве параметра ей желательно передавать только числа больше, либо равные нулю. Для всех параметров меньше нуля, функция Lg возвращает [-BigNum](BigNum.md). |
| [limit](../help/topics/Limit.md) | `FLOAT limit(FLOAT, FLOAT, FLOAT)` | 871 | Функция позволяет ограничить значение аргумента value в заданном интервале [min,max] |
| [ln](../help/topics/Ln.md) | `FLOAT ln(FLOAT)` | 29 | Функция вычисляет натуральный логарифм от переданного ей аргумента. В качестве параметра ей желательно передавать только числа больше, либо равные нулю. Для всех параметров меньше нуля, функция ln возвращает [-BigNum](BigNum.md). |
| [log](../help/topics/Log.md) | `FLOAT log(FLOAT, FLOAT)` | 31 | Функция Log вычисляет логарифм данного числа по данному основанию. Число, для которого вычисляется логарифм, и основание этого логарифма должны быть положительными числами, в противном случае функция Log вернет [-BigNum](BigNum.md). |
| [max](../help/topics/Max.md) | `FLOAT max(FLOAT, FLOAT)` | 38 | Функция находит максимальное из двух передаваемых ей чисел |
| [min](../help/topics/Min.md) | `FLOAT min(FLOAT, FLOAT)` | 39 | Функция Min находит минимальное из двух передаваемых ей чисел |
| [not](../help/topics/Not.md) | `FLOAT not(FLOAT)`<br>`FLOAT not(HANDLE)` | 45, 87 | Функция Not выполняет логическую операцию “не” над своим аргументом. |
| [NotBin](../help/topics/NotBin.md) | `FLOAT NotBin(FLOAT)` | 88 | Функция осуществляет побитовую операцию НЕ над аргументом. |
| [or](../help/topics/Or.md) | `FLOAT or(FLOAT, FLOAT)` | 44 | Функция выполняет логическую операцию “или” над своими аргументами. |
| [rad](../help/topics/rad.md) | `FLOAT rad(FLOAT)` | 48 | Функция преобразует значение угла из градусов в радианы (PI*arg/180). |
| [rnd](../help/topics/Rnd.md) | `FLOAT rnd(FLOAT)` | 42 | Функция генерирует случайное число, равномерно распределенное в диапазоне 0 - arg |
| [round](../help/topics/Round.md) | `FLOAT round(FLOAT, FLOAT)` | 41 | Функция Round округляет число arg1 с точностью, определяемой arg2. |
| [sgn](../help/topics/Sgn.md) | `FLOAT sgn(FLOAT)` | 47 | Функция вычисляет сигнум - функцию от своего аргумента. |
| [sin](../help/topics/Sin.md) | `FLOAT sin(FLOAT)` | 25 | Функция Sin вычисляет значение синуса угла, значение которого передано ей в качестве параметра. Угол должен быть задан в радианах. |
| [sqr](../help/topics/Sqr.md) | `FLOAT sqr(FLOAT)` | 35 | Функция вычисляет квадрат аргумента. |
| [sqrt](../help/topics/Sqrt.md) | `FLOAT sqrt(FLOAT)` | 34 | Функция Sqrt вычисляет квадратный корень от переданного ей аргумента. В качестве параметра ей желательно передавать только числа больше или равные нулю. Для всех параметров меньше нуля, функция Sqrt возвращает ноль. |
| [tan](../help/topics/Tan.md) | `FLOAT tan(FLOAT)` | 23 | Функция Tan вычисляет значение тангенса угла, значение которого передано ей в качестве параметра. Угол должен быть задан в радианах. |
| [trunc](../help/topics/trunc.md) | `FLOAT trunc(FLOAT)` | 80 | Функция отбрасывает дробную часть аргумента. |
| [Xor](../help/topics/Xor.md) | `FLOAT Xor(FLOAT, FLOAT)` | 97 | Функция выполняет логическую операцию XOR над своими аргументами. |
| [XorBin](../help/topics/XorBin.md) | `FLOAT XorBin(FLOAT, FLOAT)` | 98 | Функция осуществляет побитовую операцию XOR над аргументами. |

## Функции работы с имиджами (34)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [CloseAll](../help/topics/CloseAll.md) | `CloseAll()` | 81 | Функция завершает выполнение проекта. |
| [CloseClassScheme](../help/topics/CloseClassScheme.md) | `FLOAT CloseClassScheme(STRING)` | 415 | Функция закрывает окно схемы класса. |
| [CreateClass](../help/topics/CreateClass.md) | `FLOAT CreateClass(STRING, STRING, FLOAT)` | 412 | Функция создает новый класс имиджей. В качестве прототипа можно задать уже существующий класс, в этом случае новый класс будет его точной копией. Если в качестве имени класса-прототипа задается пустая строка, то создаваемый класс пустой, то есть не содержит математической модели и подсхемы. |
| [CreateLink](../help/topics/CreateLink.md) | `HANDLE CreateLink(STRING, HANDLE, HANDLE)`<br>`HANDLE CreateLink(STRING, STRING, STRING)` | 416, 436 | Функция создает связь между двумя объектами. Связь создается пустая, для добавления в нее переменных необходимо использовать функцию [SetLinkVars](SetLinkVars.md). |
| [CreateObject](../help/topics/CreateObject.md) | `HANDLE CreateObject(STRING, STRING, STRING, FLOAT, FLOAT)` | 410 | Функция добавляет в схему заданного класса новый имидж. |
| [DeleteClass](../help/topics/DeleteClass.md) | `DeleteClass(STRING)` | 413 | Функция удаляет описание указанного класса. Не выполняется в случае использования в проекте объектов данного класса. |
| [DeleteObject](../help/topics/DeleteObject.md) | `FLOAT DeleteObject(STRING, HANDLE, FLOAT)` | 411 | Функция позволяет удалить из схемы некоего класса указанный имидж. |
| [Exit](../help/topics/Exit.md) | `Exit()` | 0 | Функция Exit прерывает процесс обработки математической модели имиджа. Далее вычисляется следующий имидж. |
| [GetCalcOrder](../help/topics/GetCalcOrder.md) | `FLOAT GetCalcOrder(STRING, HANDLE)` | 256 | Функция позволяет определить порядок вычисления определенного имиджа. |
| [GetClassFile](../help/topics/GetClassFile.md) | `STRING GetClassFile(STRING)` | 1110 | Функция возвращает имя файла заданного класса без пути и расширения |
| [GetClassName](../help/topics/GetClassName.md) | `STRING GetClassName(STRING)` | 398 | Функция возвращает класс объекта по его имени. |
| [GethObjectByName](../help/topics/GetHObjectByName.md) | `HANDLE GethObjectByName(STRING)` | 420 | Функция возвращает дескриптор объекта по его имени. |
| [GetHObjectByNum](../help/topics/GetHObjectByNum.md) | `HANDLE GetHObjectByNum(STRING, FLOAT)` | 387 | Функция возвращает дескриптор имиджа по его номеру. |
| [GetLink](../help/topics/GetLink.md) | `HANDLE GetLink(STRING, HANDLE, HANDLE)` | 607 | Функция возвращает дескриптор связи между двумя объектами. |
| [GetModelText](../help/topics/GetModelText.md) | `FLOAT GetModelText(STRING, HANDLE)` | 438 | Функция считывает в указанный поток текст математической модели указанного класса. Поток необходимо предварительно создать. |
| [GetNameByHandle](../help/topics/GetNameByHandle.md) | `STRING GetNameByHandle(STRING, HANDLE)` | 486 | Функция позволяет определить имя имиджа на родительской схеме. |
| [GetObjectClass](../help/topics/GetObjectClass.md) | `STRING GetObjectClass(STRING, HANDLE)` | 389 | Функция возвращает класс объекта по его дескриптору. |
| [GetObjectCount](../help/topics/GetObjectCount.md) | `FLOAT GetObjectCount(STRING)` | 386 | Функция возвращает число объектов в схеме класса. |
| [GetProjectClasses](../help/topics/GetProjectClasses.md) | `HANDLE GetProjectClasses(FLOAT)` | 1109 | Функция возвращает список имен классов текущего проекта. |
| [GetUniqueClassName](../help/topics/GetUniqueClassName.md) | `STRING GetUniqueClassName(STRING)` | 419 | Функция возвращает уникальное имя класса с использованием базовой строки. |
| [GetVarS](../help/topics/GetVar.md) | `STRING GetVarS(STRING, STRING)` | 431 | Функция возвращает значение переменной указанного объекта. |
| [LoadObjectState](../help/topics/LoadObjectState.md) | `FLOAT LoadObjectState(STRING, STRING)` | 496 | Функция считывает из файла состояние переменных для указанного имиджа. |
| [OpenClassScheme](../help/topics/OpenClassScheme.md) | `HANDLE OpenClassScheme(STRING, FLOAT)` | 414 | Функция открывает схему указанного класса. |
| [Quit](../help/topics/Quit.md) | `Quit(FLOAT)` | 107 | Функция завершает работу проекта. Происходит сохранение всех данных проекта и выход из среды **Stratum**. |
| [RemoveLink](../help/topics/RemoveLink.md) | `FLOAT RemoveLink(STRING, HANDLE)` | 418 | Функция удаляет связь между двумя объектами. |
| [SaveObjectState](../help/topics/SaveObjectState.md) | `FLOAT SaveObjectState(STRING, STRING)` | 497 | Функция сохраняет в файле состояние переменных указанного имиджа. |
| [SendMessage](../help/topics/SendMessage.md) | `SendMessage(STRING, STRING, [STRING, STRING])` | 600 | Функция осуществляет посылку сообщения объекту или группе объектов (см. [Механизм сообщений](Message_theorethic.md)). Функция имеет переменное число аргументов для возможности указания произвольного числа связываемых переменных. |
| [SetCalcOrder](../help/topics/SetCalcOrder.md) | `FLOAT SetCalcOrder(STRING, HANDLE, FLOAT)` | 255 | Функция позволяет определить порядок вычисления определенного имиджа. |
| [SetLinkVars](../help/topics/SetLinkVars.md) | `FLOAT SetLinkVars(STRING, HANDLE, STRING)` | 417 | Функция устанавливает в связь между объектами пару связываемых переменных. Переменные, установленные в этой связи, до вызова данной функции, уничтожаются. |
| [SetModelText](../help/topics/SetModelText.md) | `FLOAT SetModelText(STRING, HANDLE)`<br>`FLOAT SetModelText(STRING, HANDLE, FLOAT)` | 437, 1116 | Функция устанавливает новый текст математической модели. После чего происходит автоматическая компиляция текста и, если не произошло ошибок, то он устанавливается в класс. Если дескриптор потока нулевой или неверный, то в класс устанавливается пустая модель. |
| [SetObjectName](../help/topics/SetObjectName.md) | `FLOAT SetObjectName(STRING, HANDLE, STRING)` | 601 | Функция позволяет установить имя имиджа на родительской схеме. |
| [SetVar](../help/topics/SetVar.md) | `SetVar(STRING, STRING, FLOAT)`<br>`SetVar(STRING, STRING, STRING)`<br>`SetVar(STRING, STRING, HANDLE)`<br>`SetVar(STRING, STRING, COLORREF)` | 433, 434, 435, 435 | Функция устанавливает значение переменной в указанном объекте. |
| [SetVarsToDefault](../help/topics/SetVarsToDefault.md) | `FLOAT SetVarsToDefault(STRING)` | 492 | Функция устанавливает переменные указанного имиджа в значения заданные по умолчанию. |
| [Stop](../help/topics/Stop.md) | `Stop(FLOAT)` | 50 | Если флаг положительный, то функция останавливает процесс вычисления схемы, то есть вместо |

## Функции работы с окнами (30)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [ArrangeIcons](../help/topics/ArrangeIcons.md) | `ArrangeIcons()` | 218 | Функция упорядочивает иконки, расположенные на рабочей поверхности главного окна среды. |
| [BringWindowToTop](../help/topics/BringWindowToTop.md) | `FLOAT BringWindowToTop(STRING)` | 215 | Функция помещает окно поверх всех остальных. |
| [CascadeWindows](../help/topics/CascadeWindows.md) | `CascadeWindows()` | 216 | Функция располагает окна каскадом, окна частично перекрываются. |
| [CloseWindow](../help/topics/CloseWindow.md) | `FLOAT CloseWindow(STRING)` | 202 | Функция закрывает окно с заданным именем. |
| [DbBrowse](../help/topics/DbBrowse.md) | `FLOAT DbBrowse(STRING, HANDLE, STRING)` | 515 | Функция осуществляет просмотр и редактирование базы данных в окне. |
| [GetClientHeight](../help/topics/GetClientHeight.md) | `FLOAT GetClientHeight(STRING)` | 226 | Функция возвращает высоту клиентской части окна (размер видимой области пространства). |
| [GetClientWidth](../help/topics/GetClientWidth.md) | `FLOAT GetClientWidth(STRING)` | 225 | Функция возвращает ширину клиентской части окна (размер видимой области пространства). |
| [GetWindowHeight](../help/topics/GetWindowHeight.md) | `FLOAT GetWindowHeight(STRING)` | 222 | Функция возвращает высоту окна. |
| [GetWindowName](../help/topics/GetWindowName.md) | `STRING GetWindowName(HANDLE)` | 203 | Функция возвращает имя окна по дескриптору графического пространства. |
| [GetWindowOrgX](../help/topics/GetWindowOrgX.md) | `FLOAT GetWindowOrgX(STRING)` | 219 | Функция возвращает X координату левой верхней точки окна. |
| [GetWindowOrgY](../help/topics/GetWindowOrgY.md) | `FLOAT GetWindowOrgY(STRING)` | 220 | Функция возвращает Y координату левой верхней точки окна. |
| [GetWindowProp](../help/topics/GetWindowProp.md) | `STRING GetWindowProp(STRING, STRING)` | 498 | Функция позволяет получить значение внутреннего параметра заданного окна. |
| [GetWindowSpace](../help/topics/GetWindowSpace.md) | `HANDLE GetWindowSpace(STRING)` | 204 | Функция возвращает дескриптор графического пространства, расположенного в указанном окне. |
| [GetWindowTitle](../help/topics/GetWindowTitle.md) | `STRING GetWindowTitle(STRING)` | 207 | Функция возвращает заголовок окна. |
| [GetWindowWidth](../help/topics/GetWindowWidth.md) | `FLOAT GetWindowWidth(STRING)` | 221 | Функция возвращает ширину окна. |
| [IsIconic](../help/topics/IsIconic.md) | `FLOAT IsIconic(STRING)` | 213 | Функция определяет - минимизировано окно или нет. |
| [IsWindowExist](../help/topics/IsWindowExist.md) | `FLOAT IsWindowExist(STRING)` | 214 | Функция определяет - существует ли окно с указанным именем. |
| [IswindowVisible](../help/topics/IsWindowVisible.md) | `FLOAT IswindowVisible(STRING)` | 212 | Функция определяет - видимо ли окно с указанным именем. |
| [SetClientSize](../help/topics/SetClientSize.md) | `FLOAT SetClientSize(STRING, FLOAT, FLOAT)` | 205 | Функция изменяет размеры окна так, чтобы размер его рабочей области был равен заданным значениям. |
| [SetScrollRange](../help/topics/SetScrollRange.md) | `FLOAT SetScrollRange(STRING, FLOAT, FLOAT, FLOAT)` | 241 | Функция устанавливает диапазон прокрутки указанного скроллера окна. |
| [SetWindowOrg](../help/topics/SetWindowOrg.md) | `FLOAT SetWindowOrg(STRING, FLOAT, FLOAT)` | 210 | Функция перемещает окно. |
| [SetWindowOwner](../help/topics/SetWindowOwner.md) | `FLOAT SetWindowOwner(HANDLE, HANDLE)` | 1107 | Функция устанавливает владельца окна (не родителя). Владелец окна будет находиться всегда под окном-потомком. Т.е. HSpace будет всегда поверх HSpaceParent. |
| [SetWindowPos](../help/topics/SetWindowPos.md) | `FLOAT SetWindowPos(STRING, FLOAT, FLOAT, FLOAT, FLOAT)` | 209 | Функция перемещает окно и устанавливает новые размеры. |
| [SetWindowRegion](../help/topics/SetWindowRegion.md) | `FLOAT SetWindowRegion(STRING, HANDLE)`<br>`FLOAT SetWindowRegion(HANDLE, HANDLE)` | 784, 795 | Функция устанавливает регион окна. |
| [SetWindowSize](../help/topics/SetWindowSize.md) | `FLOAT SetWindowSize(STRING, FLOAT, FLOAT)` | 211 | Функция изменяет размеры окна. |
| [SetWindowTitle](../help/topics/SetWindowTitle.md) | `FLOAT SetWindowTitle(STRING, STRING)` | 206 | Функция устанавливает новый заголовок окна. |
| [SetWindowTransparent](../help/topics/SetWindowTransparent.md) | `FLOAT SetWindowTransparent(STRING, FLOAT)`<br>`FLOAT SetWindowTransparent(HANDLE, FLOAT)` | 782, 793 | Функция устанавливает уровень прозрачности окна. |
| [SetWindowTransparentColor](../help/topics/SetWindowTransparentColor.md) | `FLOAT SetWindowTransparentColor(STRING, COLORREF)`<br>`FLOAT SetWindowTransparentColor(HANDLE, COLORREF)` | 783, 794 | Функция устанавливает цвет окна, который не будет отображаться в окне, т.е. окно в местах, где находится этот цвет, будет полностью прозрачным. |
| [ShowWindow](../help/topics/ShowWindow.md) | `FLOAT ShowWindow(STRING, FLOAT)` | 208 | Функция устанавливает атрибуты окна - спрятать, показать, максимизировать, минимизировать и восстановить окно. |
| [Tile](../help/topics/Tile.md) | `Tile(FLOAT)` | 217 | Функция располагает окна черепицей. |

## Функции работы с матрицами (30)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [MAddC](../help/topics/MAddC.md) | `FLOAT MAddC(FLOAT, FLOAT, FLOAT, FLOAT)` | 75 | Функция складывает две матрицы Q1 и Q2 и помещает результат в новую матрицу - Q3. То есть каждый элемент из матрицы Q1 с координатами I,J складывается с элементом из матрицы Q2 с координатами I,J , а результат помещается в элемент матрицы Q3 с координатами I,J. Если до выполнения операции существовала матрица с номером Q3, то она уничтожается.Функция выполняется, если значение флага больше или равно 1 |
| [MAddX](../help/topics/MAddX.md) | `FLOAT MAddX(FLOAT, FLOAT, FLOAT)` | 67 | Функция прибавляет к каждому элементу матрицы константу, если значение флага больше или равно 1. |
| [MCreate](../help/topics/MCreate.md) | `FLOAT MCreate(FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 60 | Функция cоздает матрицу размерами (MaxI-MinI+1) на (MaxJ-MinJ+1), если Flag больше или равен 1, иначе - создания нет. Если матрица с данным номером уже существует, то старая  матрица будет уничтожена. |
| [MCut](../help/topics/MCut.md) | `FLOAT MCut(FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 93 | Функция осуществляет "вырезку" из матрицы Q1 подматрицы Q2. Из матрицы Q1 "вырезается" матрица Q2, начиная с координат I,J, размерами dI на dJ. Функция выполняется, если значение флага больше или равно 1 |
| [MDelete](../help/topics/MDelete.md) | `FLOAT MDelete(FLOAT, FLOAT)` | 61 | Функция удаляет матрицу с указанным номером, если значение флага больше или равно 1. |
| [MDelta](../help/topics/MDelta.md) | `FLOAT MDelta(FLOAT, FLOAT)` | 72 | Вычисляется дельта функция от каждого из элементов матрицы, и результат помещается в ту же ячейку. |
| [MDet](../help/topics/MDet.md) | `FLOAT MDet(FLOAT, FLOAT)` | 71 | Функция вычисляет определитель матрицы, если значение флага больше или равно 1. |
| [MDiag](../help/topics/MDiag.md) | `FLOAT MDiag(FLOAT, FLOAT, FLOAT)` | 66 | Функция обнуляет матрицу и заполняет ее диагональ константой. Если матрица не квадратная, то оставшийся блок заполняется нулями. Функция выполняется, если значение флага больше или равно 1 |
| [MDim](../help/topics/mdim.md) | `FLOAT MDim(FLOAT, &FLOAT, &FLOAT, &FLOAT, &FLOAT)` | 96 | Функция возвращает минимальный и максимальный дипазоны матрицы по X и Y. |
| [MDivC](../help/topics/MDivC.md) | `FLOAT MDivC(FLOAT, FLOAT, FLOAT, FLOAT)` | 90 | Функция делит матрицу Q1 на Q2 поэлементно и помещает результат в новую матрицу - Q3. То есть каждый элемент из матрицы Q1 с координатами I,J делится на элемент из матрицы Q2 с координатами I,J , а результат помещается в элемент матрицы Q3 с координатами I,J. Если существовала матрица с номером Q3, то она уничтожается. Функция выполняется, если значение флага больше или равно 1 |
| [MDivX](../help/topics/MDivX.md) | `FLOAT MDivX(FLOAT, FLOAT, FLOAT)` | 69 | Функция делит каждый элемент матрицы на константу, если значение флага больше или равно 1. |
| [MEd](../help/topics/MEd.md) | `FLOAT MEd(FLOAT, FLOAT)` | 73 | Вычисляется единичная функция от всех элементов матрицы, и результат помещается в ту же ячейку матрицы. |
| [MEditor](../help/topics/MEditor.md) | `MEditor(FLOAT, FLOAT)` | 65 | Функция инициализирует специальное окно, позволяющее редактировать указанную матрицу, если значение флага больше или равно 1. |
| [MFill](../help/topics/MFill.md) | `FLOAT MFill(FLOAT, FLOAT, FLOAT)` | 62 | Функция заполняет матрицу указанным числом, если значение флага больше или равно 1. |
| [MGet](../help/topics/MGet.md) | `FLOAT MGet(FLOAT, FLOAT, FLOAT, FLOAT)` | 63 | Функция возвращает значение указанного элемент матрицы, если значение флага больше или равно 1. |
| [MGlue](../help/topics/MGlue.md) | `FLOAT MGlue(FLOAT, FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 92 | Функция осуществляет склейку матриц Q1 и Q2 и помещает результат в новую матрицу - Q3. В матрицу Q3 помещается матрица Q1 по своим координатам, а матрица Q2 помещается в координаты, которые образуются путем сложения номеров ее элементов по I с dI, по J с dJ. Если матрицы Q1 и Q2 перекрываются, то в перекрывающихся элементах в матрице Q3 сохраняются значения матрицы Q1. Если существовала матрица с номером Q3, то она уничтожается. Функция выполняется, если значение флага больше или равно 1 |
| [MLoad](../help/topics/MLoad.md) | `FLOAT MLoad(FLOAT, STRING, FLOAT)` | 101 | Функция восстанавливает матрицу из текстового файла. Функция выполняется, если значение флага больше или равно 1 |
| [MMove](../help/topics/MMove.md) | `FLOAT MMove(FLOAT, FLOAT, FLOAT, FLOAT)` | 94 | Функция перемещает матрицу Q в новые координаты. Номера строк матрицы сдвигаются на величину dI. Номера столбцов матрицы сдвигаются на величину dJ. Функция выполняется, если значение флага больше или равно 1 |
| [MMul](../help/topics/MMul.md) | `FLOAT MMul(FLOAT, FLOAT, FLOAT, FLOAT)` | 91 | Функция перемножает матрицы Q1 и Q2 по правилу "строка" на "столбец" и помещает результат в новую матрицу - Q3. |
| [MMulC](../help/topics/MMulC.md) | `FLOAT MMulC(FLOAT, FLOAT, FLOAT, FLOAT)` | 79 | Функция перемножает две матрицы Q1 и Q2 поэлементно и помещает результат в новую матрицу - Q3. То есть каждый элемент из матрицы Q1 с координатами I,J умножается на элемент из матрицы Q2 с координатами I,J , а результат помещается в элемент матрицы Q3 с координатами I,J. Если существовала матрица с номером Q3, то она уничтожается. Функция выполняется, если значение флага больше или равно 1 |
| [MMulX](../help/topics/MMulX.md) | `FLOAT MMulX(FLOAT, FLOAT, FLOAT)` | 70 | Функция умножает каждый элемент матрицы на константу, если значение флага больше или равно 1. |
| [MNot](../help/topics/MNot.md) | `FLOAT MNot(FLOAT, FLOAT)` | 76 | Функция выполняет операцию NOT над каждым из элементов матрицы и результат записывается в ту же ячейку. Функция выполняется, если значение флага больше или равно 1 |
| [MObr](../help/topics/MObr.md) | `FLOAT MObr(FLOAT, FLOAT, FLOAT)` | 95 | Функция вычисляет обратную матрицу к Q1 и помещает ее в новую Q2, которая при этом создается. Рекомендуется, чтобы матрица Q1 была квадратной. Если существовала матрица с номером Q2, то она уничтожается. Функция выполняется, если значение флага больше или равно 1 |
| [MPut](../help/topics/MPut.md) | `FLOAT MPut(FLOAT, FLOAT, FLOAT, FLOAT, FLOAT)` | 64 | Функция устанавливает значение в указанный элемент матрицы, если значение флага больше или равно 1. |
| [MSaveAs](../help/topics/MSaveAs.md) | `FLOAT MSaveAs(FLOAT, STRING, FLOAT)` | 100 | Функция сохраняет матрицу в текстовом файле. Функция выполняется, если значение флага больше или равно 1 |
| [MSort](../help/topics/MSort.md) | `FLOAT MSort(FLOAT, FLOAT, FLOAT, FLOAT)` | 760 | Функция сортирует матрицу Q по указанной строке (столбцу). Функция выполняется, если значение флага больше или равно 1 |
| [MSubC](../help/topics/MSubC.md) | `FLOAT MSubC(FLOAT, FLOAT, FLOAT, FLOAT)` | 78 | Функция вычитает две матрицы Q1 и Q2 поэлементно и помещает результат в новую матрицу - Q3. То есть из каждого элемента матрицы Q1 с координатами I,J вычитается элемент матрицы Q2 с координатами I,J , а результат помещается в элемент матрицы Q3 с координатами I,J. Если существовала матрица с номером Q3, то она уничтожается.Функция выполняется, если значение флага больше или равно 1 |
| [MSubX](../help/topics/MSubX.md) | `FLOAT MSubX(FLOAT, FLOAT, FLOAT)` | 68 | Функция вычитает из каждого элемента матрицы константу, если значение флага больше или равно 1. |
| [MSum](../help/topics/MSum.md) | `FLOAT MSum(FLOAT, FLOAT)` | 77 | Функция выполняет операцию суммирования всех элементов матрицы. Функция выполняется, если значение флага больше или равно 1 |
| [MTransp](../help/topics/MTransp.md) | `FLOAT MTransp(FLOAT, FLOAT, FLOAT)` | 74 | Функция транспонирует матрицу Q1 и помещает результат в новую матрицу - Q2. То есть каждый элемент I,J матрицы Q2 становится равен элементу J,I матрицы Q1. Если существовала матрица с номером Q2, то она уничтожается.Функция выполняется, если значение флага больше или равно 1 |

## Функции работы с интерфейсными элементами (20)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [CheckDlgButton2d](../help/topics/CheckDlgButton2d.md) | `FLOAT CheckDlgButton2d(HANDLE, HANDLE, FLOAT)` | 393 | Функция устанавливает кнопки CHECKBUTTON, RADIOBUTTON, 3STATEBUTTON в одно из состояний -"включено", "выключено" (и "неопределенное" для 3STATEBUTTON) |
| [CreateControlObject2d](../help/topics/CreateControlObject2d.md) | `CreateControlObject2d(HANDLE, STRING, STRING, FLOAT)` |  | Функция создает в указанном графическом пространстве интерфейсный Windows объект (см. [интерфейсные элементы](Control_objects.md)). |
| [DbSetControlTable](../help/topics/DbSetControlTable.md) | `FLOAT DbSetControlTable(HANDLE, HANDLE, HANDLE, STRING)` | 581 |  |
| [EnableControl2d](../help/topics/EnableControl2d.md) | `FLOAT EnableControl2d(HANDLE, HANDLE, FLOAT)` | 395 | Функция устанавливает состояние интерфейсного элемента - "активирован", "неактивирован". |
| [GetControlStyle2d](../help/topics/GetControlStyle2d.md) | `FLOAT GetControlStyle2d(HANDLE, HANDLE)` | 461 | Функция позволяет получить текущий стиль указанного [интерфейсного элемента](Control_objects.md). |
| [GetControlText2d](../help/topics/GetControlText2d.md) | `STRING GetControlText2d(HANDLE, HANDLE)`<br>`STRING GetControlText2d(HANDLE, HANDLE, FLOAT, FLOAT)` | 391, 1104 | Функция возвращает текст или его часть, содержащийся в указанном [интерфейсном элементе](Control_objects.md) (только для EDIT, BUTTON, COMBOBOX). |
| [IsDlgButtonChecked2d](../help/topics/IsDlgButtonChecked2d.md) | `FLOAT IsDlgButtonChecked2d(HANDLE, HANDLE)` | 394 | Функция проверяет текущее состояние кнопок типа CHECKBUTTON, RADIOBUTTON, 3STATEBUTTON. |
| [LBAddString](../help/topics/LBAddString.md) | `FLOAT LBAddString(HANDLE, HANDLE, STRING)` | 462 | Функция добавляет строку в конец списка (только для LISTBOX и COMBOBOX). |
| [LBClearList](../help/topics/LBClearList.md) | `FLOAT LBClearList(HANDLE, HANDLE)` | 465 | Функция удаляет все строки из списка (только для LISTBOX и COMBOBOX). |
| [LBDeleteString](../help/topics/LBDeleteString.md) | `FLOAT LBDeleteString(HANDLE, HANDLE, FLOAT)` | 466 | Функция удаляет строку из списка с заданным номером (только для LISTBOX и COMBOBOX). |
| [LBFindString](../help/topics/LBFindString.md) | `FLOAT LBFindString(HANDLE, HANDLE, STRING, FLOAT)` | 472 | Функция ищет, начиная с заданного номера в списке строк, строку, содержащую указанный текст, и возвращает ее номер (только для LISTBOX и COMBOBOX). |
| [LBFindStringExact](../help/topics/LBFindStringExact.md) | `FLOAT LBFindStringExact(HANDLE, HANDLE, STRING, FLOAT)` | 473 | Функция ищет, начиная с заданного номера в списке строк, строку содержащую заданный текст, и возвращает ее номер (только для LISTBOX и COMBOBOX). |
| [LBGetCaretIndex](../help/topics/LBGetCaretIndex.md) | `FLOAT LBGetCaretIndex(HANDLE, HANDLE)` | 470 | Функция возвращает номер строки в списке, выбранной пользователем (отмечается синим цветом) (только для LISTBOX и COMBOBOX). |
| [LBGetString](../help/topics/LBGetString.md) | `STRING LBGetString(HANDLE, HANDLE, FLOAT)` | 464 | Функция возвращает строку с указанным номером из списка (только для LISTBOX и COMBOBOX). |
| [LBSetCaretIndex](../help/topics/LBSetCaretIndex.md) | `FLOAT LBSetCaretIndex(HANDLE, HANDLE, FLOAT)` | 471 | Функция выделяет (синим цветом) строку в списке с указанным номером (только для LISTBOX и COMBOBOX). |
| [LBSetSelIndex](../help/topics/LBSetSelIndex.md) | `FLOAT LBSetSelIndex(HANDLE, HANDLE, FLOAT)` | 469 | Функция выделяет (рамкой) строку в списке с указанным номером (только для LISTBOX и COMBOBOX). |
| [SetControlFocus2d](../help/topics/SetControlFocus2d.md) | `SetControlFocus2d(HANDLE, HANDLE)` | 1119 | Функция устанавливает фокус ввода в интерфейсный элемент. |
| [SetControlFont2d](../help/topics/SetControlFont2d.md) | `FLOAT SetControlFont2d(HANDLE, HANDLE, HANDLE)` | 1101 | Функция устанавливает шрифт текста в интерфейсный элемент. |
| [SetControlStyle2d](../help/topics/SetControlStyle2d.md) | `FLOAT SetControlStyle2d(HANDLE, HANDLE, FLOAT)` | 460 | Функция устанавливает объекту Windows новый стиль. |
| [SetControlText2d](../help/topics/SetControlText2d.md) | `FLOAT SetControlText2d(HANDLE, HANDLE, STRING)` | 392 | Функция устанавливает новый текст в интерфейсный элемент. (только для EDIT, BUTTON, COMBOBOX). |

## Функции работы с файлами и папками (16)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [AddSlash](../help/topics/AddSlash.md) | `STRING AddSlash(STRING)` | 238 | Функция прибавляет к строке, содержащей путь, слеш, если он не установлен. Если слэш уже установлен, ничего не происходит. |
| [CreateDir](../help/topics/CreateDir.md) | `FLOAT CreateDir(STRING)` | 730 | Функция создает новую директорию. Если указан путь, содержащий несуществующие директории, |
| [DeleteDir](../help/topics/DeleteDir.md) | `FLOAT DeleteDir(STRING)` | 731 | Функция удаляет указанную директорию, даже если она непустая. |
| [FileCopy](../help/topics/FileCopy.md) | `FLOAT FileCopy(STRING, STRING)` | 733 | Функция копирует файл. |
| [FileDelete](../help/topics/FileDelete.md) | `FLOAT FileDelete(STRING)` | 735 | Функция удаляет указанный файл. |
| [FileExist](../help/topics/FileExist.md) | `FLOAT FileExist(STRING)` | 734 | Функция определяет существует ли файл с указанным именем. |
| [FileRename](../help/topics/FileRename.md) | `FLOAT FileRename(STRING, STRING)` | 732 | Функция переименовывает файл или папку. |
| [GetClassDirectory](../help/topics/GetClassDirectory.md) | `STRING GetClassDirectory(STRING)` | 233 | Функция возвращает директорию, в которой располагается файл описания указанного класса. |
| [GetFileList](../help/topics/GetFileList.md) | `HANDLE GetFileList(STRING, FLOAT)` | 736 | Функция позволяет получить список файлов в заданной папке. |
| [GetPathFromFile](../help/topics/GetPathFromFile.md) | `STRING GetPathFromFile(STRING)` | 237 | Функция возвращает путь из полного имени файла. |
| [GetProjectDirectory](../help/topics/GetProjectDirectory.md) | `STRING GetProjectDirectory()` | 234 | Функция возвращает директорию проекта. |
| [GetROMDriveNames](../help/topics/GetROMDriveNames.md) | `STRING GetROMDriveNames()` | 881 | Функция позволяет определить список имен CD- и DVD-приводов, подключенных к компьютеру. |
| [GetStratumDirectory](../help/topics/GetStratumDirectory.md) | `STRING GetStratumDirectory()` | 239 | Функция возвращает директорию, в которой расположен Stratum. |
| [GetSystemDirectory](../help/topics/GetSystemDirectory.md) | `STRING GetSystemDirectory()` | 236 | Функция возвращает путь до директории System. |
| [GetTempDirectory](../help/topics/GetTempDirectory.md) | `STRING GetTempDirectory()` | 878 | Функция возвращает путь к директории для временных файлов, например, c:\Temp\ |
| [GetWindowsDirectory](../help/topics/GetWindowsDirectory.md) | `STRING GetWindowsDirectory()` | 235 | Функция возвращает директорию Windows. |

## Функции работы с динамическими массивами (16)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [delete](../help/topics/Delete.md) | `delete(HANDLE)` | 701 | Функция Delete освобождает место в динамической памяти, занимаемое динамическим массивом. |
| [GetVarCount](../help/topics/GetVarCount.md) | `FLOAT GetVarCount(STRING)` | 751 | Функция возвращает количество переменных в классе |
| [GetVarInfo](../help/topics/GetVarInfo.md) | `FLOAT GetVarInfo(STRING, FLOAT, &STRING, &STRING, &STRING, &STRING)`<br>`FLOAT GetVarInfo(STRING, FLOAT, &STRING, &STRING, &STRING, &STRING, &FLOAT)` | 750, 799 | Функция по заданному номеру переменной класса возвращает ее имя, тип, значение по умолчанию и описание этой переменной. Нумерация переменных начинается с нуля. |
| [new](../help/topics/New.md) | `HANDLE new()` | 700 | Функция New возвращает дескриптор динамического массива. |
| [vClearAll](../help/topics/vClearAll.md) | `vClearAll()` | 702 | Функция vClearAll удаляет все динамические массивы. Аргументов не имеет. |
| [vDelete](../help/topics/vDelete.md) | `FLOAT vDelete(HANDLE, FLOAT)` | 704 | Функция vDelete удаляет из динамического массива один элемент по его порядковому номеру. Нумерация идет с нуля. После удаления оставшиеся элементы сдвигаются. |
| [vGetCount](../help/topics/vGetCount.md) | `FLOAT vGetCount(HANDLE)` | 705 | Функция возвращает количество элементов в динамическом массиве. |
| [vGetF](../help/topics/vGetF.md) | `FLOAT vGetF(HANDLE, FLOAT, STRING)` | 707 | Функция получает значение элемента динамического массива типа FLOAT. Если элемент массива является структурой, то дополнительно указывается имя поля, если нет, то указывается пустая строка. |
| [vGetH](../help/topics/vGetH.md) | `HANDLE vGetH(HANDLE, FLOAT, STRING)` | 709 | Функция vGetH получает значение элемента динамического массива типа HANDLE. Если элемент массива является структурой, то дополнительно указывается имя поля, если нет, то указывается пустая строка. |
| [vGetS](../help/topics/vGetS.md) | `STRING vGetS(HANDLE, FLOAT, STRING)` | 708 | Функция получает значение элемента динамического массива типа STRING. Если элемент массива является структурой, то дополнительно указывается имя поля, если нет, то указывается пустая строка. |
| [vGetType](../help/topics/vGetType.md) | `STRING vGetType(HANDLE, FLOAT)` | 706 | Функция возвращает тип указанного элемента в динамическом массиве. |
| [vInsert](../help/topics/vInsert.md) | `FLOAT vInsert(HANDLE, STRING)` | 703 | Функция vInsert увеличивает динамический массив на произвольную структуру данных. Добавляемый тип даных может быть как простым (STRING, FLOAT, HANDLE), так и сложным. В последнем случае добавляется имя имиджа, являющегося прототипом структуры. |
| [vLoad](../help/topics/VLoad.md) | `HANDLE vLoad(HANDLE)` | 753 | Функция создает из указанного  [потока](Streams.md) динамический массив. |
| [vSave](../help/topics/VSave.md) | `FLOAT vSave(HANDLE, HANDLE)` | 752 | Функция сохраняет в [потоке](Streams.md) указанный динамический массив. |
| [vSet](../help/topics/vSet.md) | `vSet(HANDLE, FLOAT, STRING, FLOAT)`<br>`vSet(HANDLE, FLOAT, STRING, STRING)`<br>`vSet(HANDLE, FLOAT, STRING, HANDLE)` | 710, 711, 712 | Устанавливает значение в указанный элемент динамического массива. Если элемент является структурой, то дополнительно указывается имя поля |
| [vSort](../help/topics/vSort.md) | `FLOAT vSort(HANDLE, FLOAT, [STRING])`<br>`FLOAT vSort(HANDLE, [STRING])`<br>`FLOAT vSort(HANDLE, [STRING, FLOAT])` | 873, 874, 875 | Сортирует массив по заданному списку полей. Если массив имеет базовый тип (FLOAT, STRING…), то имена указывать не нужно. Сортировка по нескольким полям полезна тогда, когда в массиве есть элементы с одинаковым значением по одному полю и с различным значением по другому. |

## Функции работы с потоками (15)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [CloseStream](../help/topics/CloseStream.md) | `FLOAT CloseStream(HANDLE)` | 162 | Функция закрывает открытый поток. |
| [CopyBlock](../help/topics/CopyBlock.md) | `FLOAT CopyBlock(HANDLE, HANDLE, FLOAT, FLOAT)` | 175 | Функция копирует содержимое одного потока в другой. |
| [CreateStream](../help/topics/CreateStream.md) | `HANDLE CreateStream(STRING, STRING, STRING)` | 161 | Функция создает поток. |
| [Eof](../help/topics/Eof.md) | `FLOAT Eof(HANDLE)` | 165 | Функция определяет - является ли текущая позиция в потоке последней из возможных. |
| [GetLine](../help/topics/GetLine.md) | `STRING GetLine(HANDLE, FLOAT, STRING)` | 174 | Функция возвращает считанную из потока строку с текущей позиции до заданного символа, но не более указанного размера. |
| [Getpos](../help/topics/GetPos.md) | `FLOAT Getpos(HANDLE)` | 167 | Функция возвращает текущую позицию в потоке. |
| [GetSize](../help/topics/GetSize.md) | `FLOAT GetSize(HANDLE)` | 168 | Функция возвращает текущий размер потока. |
| [Read](../help/topics/Read.md) | `FLOAT Read(HANDLE)` | 170 | Функция читает из потока число с текущей позиции и с текущей [шириной](width_constans.md). При этом происходит автоматическое преобразование любых типов в FLOAT. |
| [ReadLn](../help/topics/Readln.md) | `STRING ReadLn(HANDLE)` | 171 | Функция читает из потока строку с текущей позиции до символа перевода строки. |
| [Seek](../help/topics/Seek.md) | `FLOAT Seek(HANDLE, FLOAT)` | 163 | Функция устанавливает текущую позицию в потоке. |
| [SetWidth](../help/topics/SetWidth.md) | `FLOAT SetWidth(HANDLE, FLOAT)` | 169 | Функция устанавливает текущую ширину в потоке. |
| [StreamStatus](../help/topics/StreamStatus.md) | `FLOAT StreamStatus(HANDLE)` | 164 | Функция определяет состояние потока. |
| [Truncate](../help/topics/Truncate.md) | `FLOAT Truncate(HANDLE)` | 176 | Функция удаляет содержимое потока, начиная с текущей позиции. |
| [Write](../help/topics/Write.md) | `FLOAT Write(HANDLE, FLOAT)` | 172 | Функция записывает число в поток с текущей позиции и с текущей [шириной](width_constans.md). При этом происходит автоматическое преобразование типов из типа FLOAT в тип, определяемый шириной потока. |
| [WriteLn](../help/topics/Writeln.md) | `FLOAT WriteLn(HANDLE, STRING)` | 173 | Функция записывает в поток строку. |

## Функции работы со строками (11)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [Alltrim](../help/topics/Alltrim.md) | `STRING Alltrim(STRING)` | 139 | Функция удаляет ведущие и хвостовые пробелы в строке. |
| [Ansi_To_Oem](../help/topics/Ansi_to_oem.md) | `STRING Ansi_To_Oem(STRING)` | 132 | Функция преобразует входную строку из ANSI (Windows) кодировки в OEM (Dos) кодировку. |
| [ASCII](../help/topics/Ascii.md) | `FLOAT ASCII(STRING)` | 140 | Функция преобразует первый символ строки в код |
| [Chr](../help/topics/Chr.md) | `STRING Chr(FLOAT)` | 141 | Функция преобразует код в символ. |
| [Comparei](../help/topics/Comparei.md) | `FLOAT Comparei(STRING, STRING, FLOAT)` | 135 | Функция сравнивает n первых символов переданных ей строк и возвращает: |
| [Left](../help/topics/Left.md) | `STRING Left(STRING, FLOAT)` | 125 | Функция возвращает n первых символов из строки s1. |
| [Length](../help/topics/Length.md) | `FLOAT Length(STRING)` | 136 | Функция определяет и возвращает длину строки. |
| [Oem_To_Ansi](../help/topics/Oem_to_ansi.md) | `STRING Oem_To_Ansi(STRING)` | 133 | Функция преобразует строку s1 из OEM (Dos) кодировки в ANSI (Windows) кодировку. |
| [Pos](../help/topics/Pos.md) | `FLOAT Pos(STRING, STRING, FLOAT)` | 128 | Функция возвращает позицию **n** - ого вхождения строки **s2** в строке **s1**. |
| [Rtrim](../help/topics/Rtrim.md) | `STRING Rtrim(STRING)` | 138 | Функция удаляет пробелы в конце строки. |
| [Substr](../help/topics/Substr.md) | `STRING Substr(STRING, FLOAT, FLOAT)`<br>`STRING Substr(STRING, FLOAT)` | 127, 779 | Функция возвращает n символов из переданной ей строки str, начиная с позиции pos. |

## Функции работы с анализатором текста (11)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [FindNextWord](../help/topics/FindNextWord.md) | `STRING FindNextWord(STRING)` | 893 | Функция возвращает слово, идущее в базе русского языка следующим. |
| [FindPrevWord](../help/topics/FindPrevWord.md) | `STRING FindPrevWord(STRING)` | 894 | Функция возвращает слово, идущее в базе русского языка предыдущим. |
| [GetAnswer](../help/topics/GetAnswer.md) | `STRING GetAnswer(STRING, STRING)` | 832 | Функция возвращает ответ на вопрос, заданный по предложению Sentance. |
| [GetWordForm](../help/topics/GetWordForm.md) | `STRING GetWordForm(STRING, FLOAT, STRING)` | 838 | Функция возвращает слова определенной формы, которые удовлетворяют значениям свойств FormProp. Свойства описываются через запятую следующим образом: <название свойства>:<значение свойства>,…,<название свойства>:<значение свойства>. |
| [GetWordFormCount](../help/topics/GetWordFormCount.md) | `FLOAT GetWordFormCount(STRING)` | 837 | Функция возвращает количество форм, равное количеству лексических значений слова, которые имеются в базе русского языка. |
| [GetWordInfo](../help/topics/GetWordInfo.md) | `STRING GetWordInfo(STRING)`<br>`GetWordInfo(STRING, HANDLE)` | 831, 840 | Функция возвращает полную информацию о слове в виде формы: |
| [GetWordInSentByRole](../help/topics/GetWordInSentByRole.md) | `STRING GetWordInSentByRole(STRING, STRING)` | 836 | Функция возвращает значение слова по его роли в предложении. |
| [GetWordProperty](../help/topics/GetWordProperty.md) | `STRING GetWordProperty(STRING, STRING, STRING)` | 834 | Функция возвращает все возможные свойства слова. Параметр Filter ограничивает поиск свойств конкретной частью речи. Если он пустой, то свойства ищутся во всех частях. |
| [GetWordPropertyInSent](../help/topics/GetWordPropertyInSent.md) | `STRING GetWordPropertyInSent(STRING, STRING, FLOAT, STRING)` | 835 | Функция возвращает значение указанного свойства слова в предложении. |
| [InitAnalyzer](../help/topics/InitAnalyzer.md) | `FLOAT InitAnalyzer(STRING, STRING, STRING)` | 896 | Функция для инициализации анализатора текста. Выполняет подключение к базе MySql, которая устанавливается из дополнительного дистрибутива |
| [SearchWords](../help/topics/SearchWords.md) | `HANDLE SearchWords(STRING)`<br>`HANDLE SearchWords(STRING, FLOAT)`<br>`HANDLE SearchWords(STRING, FLOAT, FLOAT)` | 839, 891, 892 | Функция возвращает слова из базы, удовлетворяющие критерию поиска SearchCriteria. В критерии нужно указать искомое слово или его часть с использованием регулярных выражений (см. [Синтаксис регулярных выражений анализатора текста](regular_syntax.md)). Также можно задавать свойства слова. |

## Функции работы со стандартными диалогами (5)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [ChoseFolderDialog](../help/topics/ChoseFolderDialog.md) | `STRING ChoseFolderDialog(STRING, STRING, FLOAT)` | 99 | Запрос на выбор каталога. |
| [FileLoadDialog](../help/topics/FileLoadDialog.md) | `STRING FileLoadDialog(STRING, STRING, STRING)` | 103 | Функция инициализирует диалог открытия файла, позволяющий ввести строку текста, соответствующую выбранному файлу. |
| [FileSaveDialog](../help/topics/FileSaveDialog.md) | `STRING FileSaveDialog(STRING, STRING, STRING)` | 102 | Функция инициализирует диалог сохранения файла, позволяющий ввести строку текста, соответствующую выбранному файлу. |
| [InputBox](../help/topics/InputBox.md) | `STRING InputBox(STRING, STRING, STRING)` | 159 | Функция инициализирует диалог, позволяющий ввести строку текста. |
| [MessageBox](../help/topics/MessageBox.md) | `FLOAT MessageBox(STRING, STRING, FLOAT)` | 160 | Функция инициализирует информационный диалог. |

## Функции работы с сообщениями (4)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [RegisterObject](../help/topics/RegisterObject.md) | `RegisterObject(HANDLE, HANDLE, STRING, FLOAT, FLOAT)`<br>`RegisterObject(STRING, HANDLE, STRING, FLOAT, FLOAT)` | 153, 248 | Функция позволяет зарегистрировать объект на приемку заданного WINDOWS сообщения. |
| [ReleaseCapture](../help/topics/ReleaseCapture.md) | `ReleaseCapture()` | 156 | Функция устанавливает обычный режим обработки сообщений. Вызывается для отмены действий функции SetCapture(). |
| [SetCapture](../help/topics/SetCapture.md) | `SetCapture(HANDLE, STRING, FLOAT)` | 155 | Функция устанавливает прием сообщений от мышки и клавиатуры в заданный объект и соответствующее окно. |
| [UnregisterObject](../help/topics/UnRegisterObject.md) | `UnregisterObject(HANDLE, STRING, FLOAT)`<br>`UnregisterObject(STRING, STRING, FLOAT)` | 154, 249 | Функция изолирует объект от приема определенных сообщений от Windows. |

## Функции преобразования типов (3)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [HANDLE](../help/topics/HANDLE.md) | `HANDLE HANDLE(FLOAT)` | 16 | Функция преобразует переданное ей число в дескриптор. |
| [RGBf](../help/topics/RGBf.md) | `COLORREF RGBf(FLOAT)` | 16 | Функция позволяет перевести число из формата FLOAT в формат COLORREF. |
| [String](../help/topics/String.md) | `STRING String(FLOAT)` | 142 | Функция преобразует переданное ей число в строку. |

## Функции работы с сетью (1)

| Функция | Сигнатуры | Опкод | Описание |
|---|---|---|---|
| [RegisterNetObject](../help/topics/UnRegisterNetObject.md) | `FLOAT RegisterNetObject(STRING Object)` |  | Функция позволяет отменить соединение имиджа на локальной машине c имиджем на удаленной машине. |

