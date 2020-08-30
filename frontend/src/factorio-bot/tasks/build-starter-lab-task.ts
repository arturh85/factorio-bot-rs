import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {countEntitiesFromBlueprint} from "@/factorio-bot/util";
import {Direction, FactorioEntity} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {blueprintTileableStarterScience, blueprintTileableStarterSteamEngineBoiler} from "@/factorio-bot/blueprints";
import {createBuildBlueprint} from "@/factorio-bot/tasks/build-blueprint-task";

const TASK_TYPE = 'build-starter-lab'

type TaskData = {
    labCount: number,
    ignoreBlueprintEntities: boolean
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const offshorePumpPosition = store.state.world.starterOffshorePump
    if (!offshorePumpPosition) {
        throw new Error("lab requires offshore pump")
    }
    const offset = (store.state.world.starterScienceBlueprints || []).length
    const subtasks: Task[] = []
    for (let scienceIndex = 0; scienceIndex < data.labCount; scienceIndex++) {
        const subtask = await createBuildBlueprint(store, 'Starter Science Lab', blueprintTileableStarterScience, {
                x: offshorePumpPosition.x + 2 + (scienceIndex + offset) * 6,
                y: offshorePumpPosition.y - 18,
            },
            Direction.north, data.ignoreBlueprintEntities)
        store.commit('addSubTask', {id: task.id, task: subtask})
        subtasks.push(subtask)
    }
    let entities: FactorioEntity[] = []
    for (const subtask of subtasks) {
        const result = await executeTask(store, bots, subtask) as FactorioEntity[]
        store.commit('addStarterScienceEntities', result)
        entities = entities.concat(result)
    }
    return entities
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterLabTask(store: Store<State>, labCount: number, ignoreBlueprintEntities: boolean): Promise<Task> {
    const data: TaskData = {
        labCount,
        ignoreBlueprintEntities,
    }
    return createTask(TASK_TYPE, `Build Starter Lab x ${labCount}`, data)
}