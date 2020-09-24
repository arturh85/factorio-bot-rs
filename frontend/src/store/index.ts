import Vue from "vue";
import Vuex from "vuex";
import {findTask, Task} from "@/factorio-bot/task";

import {
    FactorioEntity,
    FactorioEntityPrototypeByName,
    FactorioForce, FactorioInventory,
    FactorioItemPrototypeByName,
    FactorioPlayer,
    FactorioPlayerById,
    FactorioRecipeByName, PlayerChangedDistanceEvent,
    PlayerChangedMainInventoryEvent,
    PlayerChangedPositionEvent, PlayerLeftEvent,
    Position,
    StarterCoalLoop,
    StarterMinerChest,
    StarterMinerFurnace,
    World
} from "@/factorio-bot/types";
import {emptyWorld, positionEqual} from "@/factorio-bot/util";

Vue.use(Vuex);

type BusyCounterByPlayerId = {[playerId: string]: number}

export type State = {
    players: FactorioPlayerById,
    busyPlayers: BusyCounterByPlayerId,
    recipes: FactorioRecipeByName,
    force: FactorioForce,
    itemPrototypes: FactorioItemPrototypeByName,
    entityPrototypes: FactorioEntityPrototypeByName,
    tasks: Task[],
    plans: string[],
    taskGraphDot: string | null,
    world: World,
    selectedTask: Task | null
}

export const initialState: State = {
    players: {},
    busyPlayers: {},
    recipes: {},
    itemPrototypes: {},
    entityPrototypes: {},
    force: {} as any,
    tasks: [],
    plans: [],
    world: emptyWorld,
    selectedTask: null,
    taskGraphDot: null,
}

export default new Vuex.Store({
    state: initialState,
    getters: {
        availablePlayers: (state: State) => (): FactorioPlayer[] => {
            return Object.keys(state.players)
                .filter(playerId => !state.busyPlayers[playerId])
                .map(playerId => state.players[playerId])
        },
        getPlayer: (state: State) => (playerId: number): FactorioPlayer => {
            return state.players[playerId.toString()]
        },
        isBusy: (state: State) => (playerId: number): boolean => {
            return !!state.busyPlayers[playerId.toString()]
        },
        getMainInventory: (state: State) => (playerId: number): FactorioInventory => {
            return state.players[playerId].mainInventory
        },
    },
    mutations: {
        changeSelectedTask(state: State, task: Task | null) {
            state.selectedTask = task
        },
        playerWorkStarted(state: State, playerId: number) {
            const strPlayerId = playerId.toString()
            const busyPlayers = {...state.busyPlayers}
            if (!busyPlayers[strPlayerId]) {
                busyPlayers[strPlayerId] = 0
            }
            busyPlayers[strPlayerId] += 1
            state.busyPlayers = busyPlayers
        },
        playerWorkFinished(state: State, playerId: number) {
            const strPlayerId = playerId.toString()
            const busyPlayers = {...state.busyPlayers}
            if (!busyPlayers[strPlayerId]) {
                return
            }
            busyPlayers[strPlayerId] -= 1
            if (busyPlayers[strPlayerId] <= 0) {
                delete busyPlayers[strPlayerId]
            }
            state.busyPlayers = busyPlayers
        },
        updateForce(state: State, force: FactorioForce) {
            state.force = force
        },
        updatePlans(state: State, plans: string[]) {
            state.plans = plans
        },
        updateEntityPrototypes(state: State, entityPrototypes: FactorioEntityPrototypeByName) {
            state.entityPrototypes = entityPrototypes
        },
        updateItemPrototypes(state: State, itemPrototypes: FactorioItemPrototypeByName) {
            state.itemPrototypes = itemPrototypes
        },
        addStarterMinerFurnace(state: State, minerFurnace: StarterMinerFurnace) {
            state.world = {...state.world, starterMinerFurnaces: [...(state.world.starterMinerFurnaces || []), minerFurnace]}
        },
        addMinerLine(state: State, params: {oreName: string, entities: FactorioEntity[]}) {
            const minerLineByOreName = {...(state.world.minerLineByOreName || {})}
            const list = minerLineByOreName[params.oreName] || []
            list.push(params.entities);
            minerLineByOreName[params.oreName] = list
            state.world = {...state.world, minerLineByOreName}
        },
        addStarterCoalLoop(state: State, coalLoop: StarterCoalLoop) {
            state.world = {...state.world, starterCoalLoops: [...(state.world.starterCoalLoops || []), coalLoop]}
        },
        addStarterMinerChest(state: State, minerChest: StarterMinerChest) {
            state.world = {...state.world, starterMinerChests: [...(state.world.starterMinerChests || []), minerChest]}
        },
        setStarterOffshorePump(state: State, pos: Position) {
            state.world = {...state.world, starterOffshorePump: pos}
        },
        addStarterSteamEngineEntities(state: State, entities: FactorioEntity[]) {
            state.world = {...state.world, starterSteamEngineBlueprints: [...(state.world.starterSteamEngineBlueprints || []), entities] }
        },
        addStarterScienceEntities(state: State, entities: FactorioEntity[]) {
            state.world = {...state.world, starterScienceBlueprints: [...(state.world.starterScienceBlueprints || []), entities] }
        },
        updateScienceGhost(state: State, newEntity: FactorioEntity) {
            const blueprints = [...(state.world.starterScienceBlueprints || [])]
            let found = false
            for (const blueprint of blueprints) {
                for(let i=0; i<blueprint.length; i++) {
                    const entity = blueprint[i]
                    if (positionEqual(entity.position, newEntity.position)) {
                        blueprint[i] = newEntity
                        found = true
                        break
                    }
                }
                if (found) {
                    break
                }
            }
            if (!found) {
                throw new Error("no entity found to update entity ghost")
            }
            state.world = {...state.world, starterScienceBlueprints: blueprints }
        },
        updateRecipes(state: State, recipes: FactorioRecipeByName) {
            state.recipes = recipes
        },
        updateTaskGraphDot(state: State, dot: string) {
            state.taskGraphDot = dot
        },
        updateWorld(state: State, world: World) {
            state.world = world
        },
        pushTask(state: State, task: Task) {
            state.tasks = [...state.tasks, task]
        },
        addSubTask(state: State, params: { id: number, task: Task }) {
            state.tasks = [...state.tasks]
            const task = findTask(state.tasks, params.id)
            if (task) {
                task.children.push(params.task)
            }
        },
        updateTask(state: State, task: Task) {
            state.tasks = [...state.tasks]
            Object.assign(findTask(state.tasks, task.id), task)
        },
        updatePlayer(state: State, player: FactorioPlayer) {
            Vue.set(state.players, player.playerId.toString(), player)
        },
        playerLeft(state: State, event: PlayerLeftEvent) {
            const _players: any = {}
            const leftPlayerId = event.playerId.toString()
            for (const playerId of Object.keys(state.players)) {
                if (playerId !== leftPlayerId) {
                    _players[playerId] = state.players[playerId]
                }
            }
            Vue.set(state, 'players', _players)
        },
        updatePlayerPosition(state: State, event: PlayerChangedPositionEvent) {
            const playerId = event.playerId.toString();
            const player: FactorioPlayer = {...(state.players[playerId] ? state.players[playerId] : {
                playerId: event.playerId,
                position: {x: 0, y: 0},
                mainInventory: {},
                buildDistance: 0,
                reachDistance: 0,
                dropItemDistance: 0,
                itemPickupDistance: 0,
                lootPickupDistance: 0,
                resourceReachDistance: 0,
            }), position: event.position}
            Vue.set(state.players, playerId, player)
        },
        updatePlayerMainInventory(state: State, event: PlayerChangedMainInventoryEvent) {
            const playerId = event.playerId.toString();
            const player: FactorioPlayer = {...(state.players[playerId] ? state.players[playerId] : {
                    playerId: event.playerId,
                    position: {x: 0, y: 0},
                    mainInventory: {},
                    buildDistance: 0,
                    reachDistance: 0,
                    dropItemDistance: 0,
                    itemPickupDistance: 0,
                    lootPickupDistance: 0,
                    resourceReachDistance: 0,
                }), mainInventory: event.mainInventory}
            Vue.set(state.players, playerId, player)
        },
        updatePlayerDistance(state: State, event: PlayerChangedDistanceEvent) {
            const playerId = event.playerId.toString();
            const player: FactorioPlayer = {...(state.players[playerId] ? state.players[playerId] : {
                    playerId: event.playerId,
                    position: {x: 0, y: 0},
                    mainInventory: {},
                    buildDistance: 0,
                    reachDistance: 0,
                    dropItemDistance: 0,
                    itemPickupDistance: 0,
                    lootPickupDistance: 0,
                    resourceReachDistance: 0,
                }),
                buildDistance: event.buildDistance,
                reachDistance: event.reachDistance,
                dropItemDistance: event.dropItemDistance,
                itemPickupDistance: event.itemPickupDistance,
                lootPickupDistance: event.lootPickupDistance,
                resourceReachDistance: event.resourceReachDistance,
            };
            Vue.set(state.players, playerId, player)
        },
        updatePlayers(state: State, players: FactorioPlayer[]) {
            const _players: any = {}
            for (const player of players) {
                _players[player.playerId.toString()] = player
            }
            Vue.set(state, 'players', _players)
        },
    },
    modules: {},
});
