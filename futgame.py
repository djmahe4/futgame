import json
import random
import codecs
from newdef import print_scorecard,determine_outcome
form1=4,4,2
form2=4,4,2

onehasball=True
goal=[]
miss=[]
save=[]
count=0
file=codecs.open("names.json",encoding='utf-8-sig')
teams=json.load(file)
keys=list(teams.keys())
team1=random.choice(keys)
keys.remove(team1)
team2=random.choice(keys)
goal_scorers=[]
players1=teams[team1]
players2=teams[team2]
game = True
file.close()

with open('desc.txt') as file:
    comments=file.readlines()
    for i in range(16):
        save.append(comments[i])
    for i in range(17, 24):
        miss.append(comments[i])
    for i in range(25, 49):
        goal.append(comments[i])

toss=input("Heads or Tails(H/T):")
ts=toss.lower()

choices = ["g","1","2","3","4","5","6","7","8","9","0"]
xG1={"g":0.01,"1":0.05,"2":0.05,"3":0.05,"4":0.05,"5":0.15,"6":0.15,"7":0.15,"8":0.15,"9":0.25,"0":0.25}
xG2={"g":0.01,"1":0.05,"2":0.05,"3":0.05,"4":0.05,"5":0.15,"6":0.15,"7":0.15,"8":0.15,"9":0.25,"0":0.25}
def print_xG(xG1,xG2):
    print(f'xG of user (wrt position):{list(xG1.values())}')
    print(f'xG of computer (wrt position):{list(xG2.values())}')
    print()
def defxG(key,xG,index):
    if index==0:
        return xG[key] + (0.01*0.8)
    elif index in range(1,5):
        return xG[key] + (0.05*0.8)
    elif index in range(5,9):
        return xG[key] + (0.15*0.8)
    elif index in range(9,11):
        return xG[key] + (0.25*0.8)
options = choices
score1 = 0
score2 = 0
counter = 90
prev=''
poss1=0
poss2=0

def check_game():
    if count==45:
        print("Half time")
        print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
        print()
        print_xG(xG1,xG2)
        global prev, options
        prev=None
        options=choices
    elif count>90:
        print("Full time")
        print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
        print()
        print_xG(xG1, xG2)
        game =False
        return True
def printsc():
    print("Team1:", score1)
    print("Team2:", score2)
    print("Kickoff!!")
    if onehasball==True:
        print("has possession")

if ts=="t" or ts=="h":
    fall = random.choice(['h','t'])
    if ts==fall:
        onehasball = True
        print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
    else:
        onehasball = False
        print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)

def selection(one):
    options=[]
    for i in range(11):
        if one=="g":
            options=["9","0","g","1","2"]
        elif one=="1":
            options = ["0", "g", "1", "2", "3"]
        elif one == "2":
            options = ["g", "1", "2", "3", "4"]
        elif one == "3":
            options = ["1", "2", "3", "4", "5"]
        elif one == "4":
            options = ["2", "3", "4", "5", "6"]
        elif one == "5":
            options = ["3", "4", "5", "6", "7"]
        elif one == "6":
            options = ["4", "5", "6", "7", "8"]
        elif one == "7":
            options = ["5", "6", "7", "8", "9"]
        elif one == "8":
            options = ["6", "7", "8", "9", "0"]
        elif one=="9":
            options = ["7", "8", "9", "0", "g"]
        elif one=="0":
            options = ["8", "9", "0", "g", "1"]
    return options

def predict(x):
    y = random.choice(x)
    return y

while game==True:
    for i in range(counter):
        count += 1
        if check_game() == True:
            game=False
            break
        elif onehasball == True:
            print("Options:", options)
            one = str(input("Enter choice:"))
            x = selection(one)
            two = predict(x)
            print("guess:", two)
            player = choices.index(one)
            ind = list(xG1.keys())[player]
            print(count, "'")
            if two == one:
                onehasball = False
                print("Possession lost!")
                poss1 = 0
                prev = one
                #break
            elif prev == one and two != one:
                chance=format(xG1[ind],".2f")
                nerves = determine_outcome(int(chance[2:4]))
                if nerves ==0:
                    print(random.choice(goal))
                    xG1[ind] = defxG(ind,xG1,player)
                    score1 += 1
                    goal_scorers.append({'team':f'{team1}',"player":f'{players1[player]}','time':f'{count}'})
                    onehasball = False
                    print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
                    poss1 = 0
                    print_xG(xG1,xG2)
                    print(f'{players1[player]}({player}) has xG :{xG1[ind]}')
                    prev=None
                    options=choices
                    break
                elif nerves == 2:
                    print(random.choice(save))
                    onehasball = True
                elif nerves == 3:
                    print(random.choice(miss))
                    print("possession lost!")
                    onehasball = False
                    prev=None
                xG1[ind] = defxG(ind,xG1,player)
                print(f'{players1[player]}({player}) has xG :{xG1[ind]}')

            else:
                if poss1 >= 10:
                    score1 += 1
                    print(random.choice(goal))
                    player = choices.index(one)
                    goal_scorers.append({'team': f'{team1}', "player": f'{players1[player]}', 'time': f'{count}'})
                    onehasball = False
                    prev=None
                    print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
                    poss1 = 0
                    xG1[ind] = defxG(ind,xG1,player)
                    print_xG(xG1,xG2)
                    print(f'{players1[player]}({player}) has xG :{xG1[ind]}')
                    options=choices
                    #iskickoff=True
                    #break
            if onehasball==False:
                break
            prev = one
            options = x
            poss1 += 1
        if onehasball == False:
            two = random.choice(options)
            print("Options:", options)
            one = str(input("Enter guess:"))
            x = selection(two)
            player = choices.index(two)
            ind = list(xG2.keys())[player]
            chance = format(xG2[ind], ".2f")
            print("attempt:", two)
            print(count, "'")
            if two == one:
                onehasball = True
                print("Possession gained!")
                poss2 = 0
                prev = two
            elif prev == two and two != one:
                nerves = determine_outcome(int(chance[2:4]))
                if nerves == 0 :
                    print(random.choice(goal))
                    xG2[ind] = defxG(ind,xG2,player)
                    print(f'{players2[player]}({player}) has xG :{xG2[ind]}')
                    score2 += 1
                    goal_scorers.append({'team': f'{team2}', "player": f'{players2[player]}', 'time': f'{count}'})
                    onehasball = True
                    prev=None
                    print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
                    poss2 = 0
                    print_xG(xG1,xG2)
                    options=choices
                    #iskickoff=True
                    break
                elif nerves == 2:
                    print(random.choice(save))
                    onehasball = False
                elif nerves == 3:
                    print(random.choice(miss))
                    print("possession gained!")
                    onehasball = True
                    prev=None
                xG2[ind] = defxG(ind,xG2,player)
                print(f'{players2[player]}({player}) has xG :{xG2[ind]}')
            else:
                if poss2 >= 10:
                    score2 += 1
                    print(random.choice(goal))
                    player = choices.index(two)
                    goal_scorers.append({'team': f'{team1}', "player": f'{players2[player]}', 'time': f'{count}'})
                    #iskickoff = True
                    onehasball = True
                    prev=None
                    print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball)
                    poss2 = 0
                    xG2[ind] = defxG(ind,xG2,player)
                    print_xG(xG1,xG2)
                    print(f'{players2[player]}({player}) has xG :{xG2[ind]}')
                    options=choices
                    #break
            if onehasball==True:
                break
            prev = two
            options = x
            poss2 += 1
