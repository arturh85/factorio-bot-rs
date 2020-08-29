import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {countEntitiesFromBlueprint} from "@/factorio-bot/util";
import {Direction, Entities, FactorioEntity, InventoryType} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {blueprintTileableStarterSteamEngineBoiler} from "@/factorio-bot/blueprints";
import {FactorioApi} from "@/factorio-bot/restApi";

const TASK_TYPE = 'build-starter-steam-engine'
const minerName = Entities.burnerMiningDrill;
const fuelName = Entities.coal;

type TaskData = {
    boilerCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    const offshorePumpPosition = store.state.world.starterOffshorePump
    if (!offshorePumpPosition) {
        throw new Error("lab requires offshore pump")
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const result = await FactorioApi.parseBlueprint(blueprintTileableStarterSteamEngineBoiler)
    const entities = countEntitiesFromBlueprint(result.blueprint)
    const subtasks: Task[] = []
    for(const name of Object.keys(entities)) {
        if (bot.mainInventory(name) < entities[name]) {
            const subtask = await createCraftTask(store, name, entities[name], false)
            store.commit('addSubTask', {id: task.id, task: subtask})
            subtasks.push(subtask)
        }
    }

    if(subtasks.length > 0) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        for (const subTask of subtasks) {
            await executeTask(store, bots, subTask)
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }

    let blueprintEntities: FactorioEntity[] = [];
    const offset = (store.state.world.starterSteamEngineBlueprints || []).length
    for (let steamIndex = 0; steamIndex < data.boilerCount; steamIndex++) {
        const blueprint = await bot.placeBlueprint(
            blueprintTileableStarterSteamEngineBoiler,
            {
                x: offshorePumpPosition.x + 2 + (steamIndex + offset) * 4,
                y: offshorePumpPosition.y - 7,
            },
            Direction.north
        );
        blueprintEntities = blueprintEntities.concat(blueprint);
        const boilers = blueprint.filter(entity => entity.name === Entities.boiler);
        for(const boiler of boilers) {
            if (bot.mainInventory(Entities.coal) > 2) {
                await bot.insertToInventory(Entities.boiler, boiler.position, InventoryType.chest_or_fuel, Entities.coal, 2)
            }
        }
        store.commit('addStarterSteamEngineEntities', blueprintEntities)
    }
    return blueprintEntities
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterSteamEngineTask(store: Store<State>, boilerCount: number): Promise<Task> {
    const data: TaskData = {
        boilerCount: boilerCount,
    }
    return createTask(TASK_TYPE, `Build Starter Steam Engine x ${boilerCount}`, data)
}