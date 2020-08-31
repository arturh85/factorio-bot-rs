import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, FactorioEntity} from "@/factorio-bot/types";
import {blueprintTileableStarterSteamEngineBoiler} from "@/factorio-bot/blueprints";
import {createBuildBlueprintTask} from "@/factorio-bot/tasks/build-blueprint-task";
import {FactorioApi} from "@/factorio-bot/restApi";

const TASK_TYPE = 'build-starter-steam-engine'

type TaskData = {
    boilerCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const offshorePumpPosition = store.state.world.starterOffshorePump
    if (!offshorePumpPosition) {
        throw new Error("steam engine requires offshore pump first")
    }
    const offset = (store.state.world.starterSteamEngineBlueprints || []).length
    const subtasks: Task[] = []
    for (let steamIndex = 0; steamIndex < data.boilerCount; steamIndex++) {
        const blueprint = await FactorioApi.parseBlueprint(blueprintTileableStarterSteamEngineBoiler, 'Starter Steam Engine')
        const subtask = await createBuildBlueprintTask(store, blueprint, {
                x: offshorePumpPosition.x + 2 + (steamIndex + offset) * blueprint.width,
                y: offshorePumpPosition.y - 7,
            },
            Direction.north, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    let entities: FactorioEntity[] = []
    for (const subtask of subtasks) {
        const result = await executeTask(store, bots, subtask) as FactorioEntity[]
        store.commit('addStarterSteamEngineEntities', result)
        entities = entities.concat(result)
    }
    return entities
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterSteamEngineTask(store: Store<State>, boilerCount: number): Promise<Task> {
    const data: TaskData = {
        boilerCount: boilerCount,
    }
    return createTask(TASK_TYPE, `Build Starter Steam Engine x ${boilerCount}`, data)
}