import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {
    availableBots,
    createTask,
    executeTask,
    Task,
    taskRunnerByType,
    TaskStatus,
    updateTaskStatus
} from "@/factorio-bot/task";
import {Direction, Entities} from "@/factorio-bot/types";
import {createBuildStarterMinerFurnaceTask} from "@/factorio-bot/tasks/build-starter-miner-furnace-task";
import {createBuildStarterMinerCoalTask} from "@/factorio-bot/tasks/build-starter-miner-coal-task";
import {createBuildStarterMinerChestTask} from "@/factorio-bot/tasks/build-starter-miner-chest-task";
import {createBuildStarterOffshorePumpTask} from "@/factorio-bot/tasks/build-starter-offshore-pump-task";
import {createBuildStarterSteamEngineTask} from "@/factorio-bot/tasks/build-starter-steam-engine-task";
import {createBuildStarterLabTask} from "@/factorio-bot/tasks/build-starter-lab-task";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {movePositionInDirection} from "@/factorio-bot/util";

const TASK_TYPE = 'build-starter-base'

type TaskData = {
    starterTargetIron: number,
    starterTargetCopper: number,
    starterTargetStone: number,
    starterTargetCoal: number,
    starterTargetLabs: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }

    if (task.status === TaskStatus.FINISHED) {
        console.warn('already finished?')
        return
    }
    // sort by already has correct item

    const addAndExecuteSubtask = async (subtask: Task): Promise<void> => {
        store.commit('addSubTask', {id: task.id, task: subtask})
        await executeTask(store, await availableBots(store), subtask)
    }
    store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
    // 1.: 2x iron miner/furnace
    if (!store.state.world.starterMinerFurnaces) {
        await addAndExecuteSubtask(await createBuildStarterMinerFurnaceTask(store, Entities.ironOre, Entities.ironPlate, Math.max(2, bots.length)))
    }
    // 2.: 2x coal miner loop
    if (!store.state.world.starterCoalLoops) {
        await addAndExecuteSubtask(await createBuildStarterMinerCoalTask(store, 2))
    }
    // 3.: 1x stone miner/chest
    if (!store.state.world.starterMinerChests) {
        await addAndExecuteSubtask(await createBuildStarterMinerChestTask(store, Entities.stone, 1))
    }
    // 4.: rest of iron miner/furnaces
    const remainingIron = data.starterTargetIron - (store.state.world.starterMinerFurnaces || []).filter(minerFurnace => minerFurnace.plateName === Entities.ironPlate).length
    await addAndExecuteSubtask(await createBuildStarterMinerFurnaceTask(store, Entities.ironOre, Entities.ironPlate, remainingIron))
    // 5.: rest of coal loops
    const remainingCoal = data.starterTargetCoal - (store.state.world.starterCoalLoops || []).length
    await addAndExecuteSubtask(await createBuildStarterMinerCoalTask(store, remainingCoal/2))
    // 6.: rest of copper miner/furnaces
    const remainingCopper = data.starterTargetCopper - (store.state.world.starterMinerFurnaces || []).filter(minerFurnace => minerFurnace.plateName === Entities.copperPlate).length
    await addAndExecuteSubtask(await createBuildStarterMinerFurnaceTask(store, Entities.copperOre, Entities.copperPlate, remainingCopper))
    // 7.: rest of stone miner/chests
    const remainingStone = data.starterTargetStone - (store.state.world.starterMinerChests || []).filter(minerChest => minerChest.oreName === Entities.stone).length
    await addAndExecuteSubtask(await createBuildStarterMinerChestTask(store, Entities.stone, remainingStone))

    // 8.: place offshore pump
    if (!store.state.world.starterOffshorePump) {
        await addAndExecuteSubtask(await createCraftTask(store, Entities.smallElectricPole, 2, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.offshorePump, 1, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.pipe, 2, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.steamEngine, 2, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.boiler, 1, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.lab, 2, false))
        await addAndExecuteSubtask(await createCraftTask(store, Entities.automationSciencePack, 10, true))
        await addAndExecuteSubtask(await createBuildStarterOffshorePumpTask(store))
    }

    if (!store.state.world.starterSteamEngineBlueprints) {
        await addAndExecuteSubtask(await createBuildStarterSteamEngineTask(store, 1))
    }

    if (!store.state.world.starterScienceBlueprints) {
        await addAndExecuteSubtask(await createBuildStarterLabTask(store, data.starterTargetLabs, true))
    }
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createBuildStarterBase(store: Store<State>, starterTargetIron: number,
                                             starterTargetCopper: number,
                                             starterTargetStone: number,
                                             starterTargetCoal: number,starterTargetLabs: number,): Promise<Task> {
    const data: TaskData = {
        starterTargetIron,
        starterTargetCopper,
        starterTargetStone,
        starterTargetCoal,
        starterTargetLabs,
    }
    return createTask(TASK_TYPE, `Build Starter Base`, data)
}