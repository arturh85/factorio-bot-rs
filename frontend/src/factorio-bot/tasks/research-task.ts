import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {createBuildStarterBase} from "@/factorio-bot/tasks/build-starter-base-task";
import {Entities, InventoryType} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {sleep} from "@/factorio-bot/util";

const TASK_TYPE = 'research'

type TaskData = {
    name: string,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const tech = store.state.force.technologies[data.name]
    if (!tech) {
        throw new Error(`tech not found: ${data.name}`)
    }
    if (tech.researched) {
        return
    }
    if (tech.prerequisites && tech.prerequisites.length > 0) {
        throw new Error("no recursion implemented yet")
    }
    const addAndExecuteSubtask = async (subtask: Task): Promise<void> => {
        store.commit('addSubTask', {id: task.id, task: subtask})
        await executeTask(store, bots, subtask)
    }
    if (!store.state.world.starterLabs) {
        await addAndExecuteSubtask(await createBuildStarterBase(store, 5, 3, 2, 2, 2))
    }
    if (!store.state.world.starterLabs || store.state.world.starterLabs.length === 0) {
        throw new Error("should have one lab?")
    }
    if (!store.state.world.starterSteamEngineBlueprints || store.state.world.starterSteamEngineBlueprints.length === 0) {
        throw new Error("should have one lab?")
    }
    for (const ingredient of tech.researchUnitIngredients) {
        if (bot.mainInventory(ingredient.name) < tech.researchUnitCount) {
            const subtask = await createCraftTask(store, ingredient.name, tech.researchUnitCount - bot.mainInventory(ingredient.name))
            store.commit('addSubTask', {id: task.id, task: subtask})
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
            await executeTask(store, bots, subtask)
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        }
    }
    await FactorioApi.addResearch(data.name)
    for (const ingredient of tech.researchUnitIngredients) {
        for (const lab of store.state.world.starterLabs) {
            await bot.insertToInventory(
                Entities.lab,
                lab,
                InventoryType.furnace_source,
                ingredient.name,
                Math.ceil(tech.researchUnitCount /  store.state.world.starterLabs.length)
            );
        }
    }
    const boilers = store.state.world.starterSteamEngineBlueprints[0].filter(entity => entity.name === Entities.boiler)
    if (boilers.length === 0) {
        throw new Error("could not find boiler?")
    }
    for(const boiler of boilers) {
        await bot.insertToInventory(
            Entities.boiler,
            boiler.position,
            InventoryType.chest_or_fuel,
            Entities.coal,
            Math.min(10, bot.mainInventory('coal'))
        );
    }
    for(let _retry = 0; _retry < tech.researchUnitCount * 3; _retry++) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        store.commit('updateForce', await FactorioApi.playerForce());
        if (store.state.force.technologies[data.name].researched) {
            break
        }
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        await sleep(10000);
    }
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createResearchTask(store: Store<State>, name: string): Promise<Task> {
    const data: TaskData = {
        name,
    }
    return createTask(TASK_TYPE, `Research ${name}`, data)
}