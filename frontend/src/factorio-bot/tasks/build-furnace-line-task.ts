import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, FactorioEntity} from "@/factorio-bot/types";
import {createBuildBlueprintTask} from "@/factorio-bot/tasks/build-blueprint-task";
import {
    blueprintFurnaceLine,
    blueprintMinerLine,
    blueprintTileableStarterSteamEngineBoiler
} from "@/blueprints/strings";
import {FactorioApi} from "@/factorio-bot/restApi";

const TASK_TYPE = 'build-furnace-line'

type TaskData = {
    oreName: string,
    plateName: string,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const firstBot = bots[0]
    const blueprint = await FactorioApi.parseBlueprint(blueprintFurnaceLine, 'Furnace Line')
    const topLeft = await firstBot.findNearestRect({x: 0, y: 0}, data.oreName, blueprint.width, blueprint.height, []);
    if(!topLeft) {
        throw new Error("failed to find ore patch big enough")
    }
    const subtask = await createBuildBlueprintTask(store, blueprint, topLeft, Direction.north, false)
    store.commit('addSubTask', {id: task.id, task: subtask})
    const entities = await executeTask(store, bots, subtask) as FactorioEntity[]
    store.commit('addMinerLine', {oreName: data.oreName, entities})
    return entities
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildFurnaceLineTask(store: Store<State>, oreName: string, plateName: string): Promise<Task> {
    const data: TaskData = {
        oreName,
        plateName,
    }
    return createTask(TASK_TYPE, `Build Furnace Line for ${oreName} -> ${plateName}`, data)
}